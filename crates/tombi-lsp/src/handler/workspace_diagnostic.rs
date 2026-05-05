use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use tombi_glob::search_pattern_matched_paths;
use tower_lsp::lsp_types::{
    FullDocumentDiagnosticReport, UnchangedDocumentDiagnosticReport, WorkspaceDiagnosticParams,
    WorkspaceDiagnosticReport, WorkspaceDiagnosticReportResult, WorkspaceDocumentDiagnosticReport,
    WorkspaceFullDocumentDiagnosticReport, WorkspaceUnchangedDocumentDiagnosticReport,
};

use crate::{
    backend::Backend,
    diagnostic::{
        DiagnosticsResult, WorkspaceConfig, get_diagnostics_result, get_workspace_configs,
    },
    document::DocumentSource,
};

#[derive(Debug, Default)]
pub struct WorkspaceDiagnosticOptions {
    pub include_open_files: bool,
}

pub async fn push_workspace_diagnostics(
    backend: &Backend,
    options: &WorkspaceDiagnosticOptions,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    log::info!("push_workspace_diagnostics");
    log::trace!("{:?}", options);

    for text_document_uri in collect_workspace_diagnostic_targets(backend).await {
        publish_workspace_diagnostics(backend, text_document_uri, options).await;
    }

    Ok(())
}

pub async fn handle_workspace_diagnostic(
    backend: &Backend,
    params: WorkspaceDiagnosticParams,
) -> Result<WorkspaceDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
    log::info!("handle_workspace_diagnostic");
    log::trace!("{:?}", params);

    let previous_result_ids = params
        .previous_result_ids
        .into_iter()
        .map(|result| (tombi_uri::Uri::from(result.uri), result.value))
        .collect::<tombi_hashmap::HashMap<_, _>>();

    let targets = collect_workspace_diagnostic_targets(backend).await;
    let target_set = targets
        .iter()
        .cloned()
        .collect::<tombi_hashmap::HashSet<_>>();
    let mut items = Vec::new();

    for text_document_uri in targets {
        let Some(diagnostics_result) = get_diagnostics_result(backend, &text_document_uri).await
        else {
            continue;
        };

        let DiagnosticsResult {
            diagnostics,
            version,
        } = diagnostics_result;
        let result_id = workspace_diagnostic_result_id(version, &diagnostics);

        if previous_result_ids
            .get(&text_document_uri)
            .is_some_and(|previous_result_id| previous_result_id == &result_id)
        {
            items.push(WorkspaceDocumentDiagnosticReport::Unchanged(
                WorkspaceUnchangedDocumentDiagnosticReport {
                    uri: text_document_uri.into(),
                    version: version.map(i64::from),
                    unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
                        result_id,
                    },
                },
            ));
            continue;
        }

        items.push(WorkspaceDocumentDiagnosticReport::Full(
            WorkspaceFullDocumentDiagnosticReport {
                uri: text_document_uri.into(),
                version: version.map(i64::from),
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: Some(result_id),
                    items: diagnostics,
                },
            },
        ));
    }

    for previous_text_document_uri in previous_result_ids.keys() {
        if target_set.contains(previous_text_document_uri) {
            continue;
        }

        items.push(WorkspaceDocumentDiagnosticReport::Full(
            WorkspaceFullDocumentDiagnosticReport {
                uri: previous_text_document_uri.clone().into(),
                version: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: Some(workspace_diagnostic_result_id(None, &[])),
                    items: Vec::new(),
                },
            },
        ));
    }

    Ok(WorkspaceDiagnosticReportResult::Report(
        WorkspaceDiagnosticReport { items },
    ))
}

fn workspace_diagnostic_result_id(
    version: Option<i32>,
    diagnostics: &[tower_lsp::lsp_types::Diagnostic],
) -> String {
    let mut hasher = DefaultHasher::new();
    version.hash(&mut hasher);
    serde_json::to_string(diagnostics)
        .unwrap_or_default()
        .hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

async fn collect_workspace_diagnostic_targets(backend: &Backend) -> Vec<tombi_uri::Uri> {
    let Some(configs) = get_workspace_configs(backend).await else {
        return Vec::new();
    };

    let mut targets = tombi_hashmap::HashSet::new();
    let home_dir = dirs::home_dir();

    for workspace_config in configs.into_values() {
        if !is_workspace_diagnostic_enabled(&workspace_config) {
            log::debug!(
                "`lsp.workspace-diagnostic.enabled` is false in {}",
                workspace_config.workspace_folder_path.display()
            );
            continue;
        }

        if let Some(home_dir) = &home_dir
            && &workspace_config.workspace_folder_path == home_dir
        {
            log::debug!(
                "Skip diagnostics for workspace folder matching $HOME: {:?}",
                workspace_config.workspace_folder_path
            );
            continue;
        }

        let files_options = workspace_config.config.files.clone().unwrap_or_default();

        for matched_path in
            search_pattern_matched_paths(workspace_config.workspace_folder_path, files_options)
                .await
        {
            let tombi_glob::FileSearchEntry::Found(path) = matched_path else {
                continue;
            };

            if let Ok(uri) = tombi_uri::Uri::from_file_path(path) {
                upsert_document_source(backend, uri.clone()).await;

                targets.insert(uri);
            }
        }
    }

    targets.into_iter().collect()
}

async fn publish_workspace_diagnostics(
    backend: &Backend,
    text_document_uri: tombi_uri::Uri,
    options: &WorkspaceDiagnosticOptions,
) {
    let Some(diagnostics_result) = get_diagnostics_result(backend, &text_document_uri).await else {
        return;
    };

    log::trace!("{:?}", diagnostics_result);

    let DiagnosticsResult {
        diagnostics,
        version,
    } = diagnostics_result;

    if !options.include_open_files && version.is_some() {
        log::debug!(
            "Skip publishing workspace diagnostics because version is some: {text_document_uri}"
        );
        return;
    }

    backend
        .client
        .publish_diagnostics(text_document_uri.into(), diagnostics, version)
        .await
}

/// Check if workspace diagnostic is enabled for the given workspace config
#[inline]
fn is_workspace_diagnostic_enabled(workspace_config: &WorkspaceConfig) -> bool {
    workspace_config
        .config
        .lsp
        .as_ref()
        .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
        .and_then(|workspace_diagnostic| workspace_diagnostic.enabled)
        .unwrap_or_default()
        .value()
}

pub async fn upsert_document_source(backend: &Backend, text_document_uri: tombi_uri::Uri) -> bool {
    let text_document_path = match text_document_uri.to_file_path() {
        Ok(text_document_path) => text_document_path,
        Err(_) => {
            log::warn!("Watcher event for non-file URI: {text_document_uri}");
            return false;
        }
    };

    let Ok(content) = tokio::fs::read_to_string(&text_document_path).await else {
        log::warn!(
            "Failed to read file for diagnostics: {:?}",
            text_document_path
        );
        return false;
    };

    let toml_version = backend
        .text_document_toml_version(&text_document_uri, &content)
        .await;
    let encoding_kind = backend.capabilities.read().await.encoding_kind;

    let mut document_sources = backend.document_sources.write().await;
    if let Some(source) = document_sources.get_mut(&text_document_uri) {
        if source.version.is_some() {
            log::debug!("Skip diagnostics for open document: {text_document_uri}");
            return false;
        }

        source.set_text(content, toml_version);
    } else {
        document_sources.insert(
            text_document_uri.clone(),
            DocumentSource::new(content, None, toml_version, encoding_kind),
        );
    }

    true
}
