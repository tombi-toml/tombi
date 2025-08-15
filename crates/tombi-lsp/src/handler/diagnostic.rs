use ahash::AHashMap;
use itertools::{Either, Itertools};
use tombi_config::LintOptions;
use tombi_file_search::FileSearch;
use tombi_uri::{url_from_file_path, url_to_file_path};
use tower_lsp::lsp_types::{
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, RelatedFullDocumentDiagnosticReport, TextDocumentIdentifier, Url,
    WorkspaceDiagnosticParams, WorkspaceDiagnosticReport, WorkspaceDiagnosticReportResult,
    WorkspaceDocumentDiagnosticReport, WorkspaceFullDocumentDiagnosticReport,
};

use crate::{
    backend::{Backend, DiagnosticType},
    config_manager::ConfigSchemaStore,
    document::DocumentSource,
};

/// Pull diagnostics
pub async fn handle_diagnostic(
    backend: &Backend,
    params: DocumentDiagnosticParams,
) -> Result<DocumentDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
    let DocumentDiagnosticParams { text_document, .. } = params;

    Ok({
        DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
            RelatedFullDocumentDiagnosticReport {
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    items: get_diagnostics_result(backend, &text_document.uri)
                        .await
                        .map(|result| result.diagnostics)
                        .unwrap_or_default(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
    })
}

/// Push diagnostics
pub async fn push_diagnostics(backend: &Backend, text_document_uri: Url, version: Option<i32>) {
    if backend.capabilities.read().await.diagnostic_type != DiagnosticType::Push {
        return;
    }

    #[derive(Debug)]
    struct PushDiagnosticsParams {
        text_document: TextDocumentIdentifier,
        version: Option<i32>,
    }

    let params = PushDiagnosticsParams {
        text_document: TextDocumentIdentifier {
            uri: text_document_uri,
        },
        version,
    };

    tracing::info!("push_diagnostics");
    tracing::trace!(?params);

    publish_diagnostics(backend, params.text_document.uri, params.version).await;
}

pub async fn handle_workspace_diagnostic(
    backend: &Backend,
    params: WorkspaceDiagnosticParams,
) -> Result<WorkspaceDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_workspace_diagnostic");
    tracing::trace!(?params);

    if let Some(workspace_diagnostic_targets) = get_workspace_diagnostic_targets(backend).await {
        let mut items = Vec::new();
        for WorkspaceDiagnosticTarget { uri, version } in workspace_diagnostic_targets {
            if let Some(diagnostics) = get_diagnostics_result(backend, &uri).await {
                items.push(WorkspaceDocumentDiagnosticReport::Full(
                    WorkspaceFullDocumentDiagnosticReport {
                        uri,
                        version: version.map(|version| version as i64),
                        full_document_diagnostic_report: FullDocumentDiagnosticReport {
                            items: diagnostics.diagnostics,
                            ..Default::default()
                        },
                    },
                ));
            }
        }

        return Ok(WorkspaceDiagnosticReportResult::Report(
            WorkspaceDiagnosticReport { items },
        ));
    }

    Ok(WorkspaceDiagnosticReportResult::Report(
        WorkspaceDiagnosticReport { items: vec![] },
    ))
}

pub async fn push_workspace_diagnostics(backend: &Backend) {
    if backend.capabilities.read().await.diagnostic_type != DiagnosticType::Push {
        return;
    }

    if let Some(workspace_diagnostic_targets) = get_workspace_diagnostic_targets(backend).await {
        for WorkspaceDiagnosticTarget { uri, version } in workspace_diagnostic_targets {
            publish_diagnostics(backend, uri, version).await;
        }
    };
}

async fn publish_diagnostics(backend: &Backend, text_document_uri: Url, version: Option<i32>) {
    let Some(DiagnosticsResult {
        diagnostics,
        version: old_version,
    }) = get_diagnostics_result(backend, &text_document_uri).await
    else {
        return;
    };

    backend
        .client
        .publish_diagnostics(text_document_uri, diagnostics, version.or(old_version))
        .await
}

struct WorkspaceDiagnosticTarget {
    uri: Url,
    version: Option<i32>,
}

async fn get_workspace_diagnostic_targets(
    backend: &Backend,
) -> Option<Vec<WorkspaceDiagnosticTarget>> {
    let workspace_folder_paths =
        backend
            .client
            .workspace_folders()
            .await
            .ok()
            .flatten()
            .map(|workspace_folders| {
                workspace_folders
                    .into_iter()
                    .filter_map(|workspace| url_to_file_path(&workspace.uri).ok())
                    .collect_vec()
            });

    tracing::debug!("workspace_folder_paths: {:?}", workspace_folder_paths);
    let Some(workspace_folder_paths) = workspace_folder_paths else {
        return None;
    };

    let mut total_diagnostic_targets = Vec::new();
    let mut configs = AHashMap::new();

    for workspace_folder_path in workspace_folder_paths {
        let Some(workspace_folder_path_str) = workspace_folder_path.to_str() else {
            continue;
        };
        if let Ok((config, config_path, config_level)) =
            serde_tombi::config::load_with_path_and_level(Some(workspace_folder_path.clone()))
        {
            configs.entry(config_path).or_insert((config, config_level));
        };
        for (config_path, (config, config_level)) in &configs {
            if let FileSearch::Files(files) = tombi_file_search::FileSearch::new(
                &[workspace_folder_path_str],
                config,
                config_path.as_deref(),
                *config_level,
            )
            .await
            {
                tracing::debug!(
                    "Found {} files, in {}",
                    files.len(),
                    workspace_folder_path_str
                );
                tracing::debug!("Founded files: {:?}", files);

                for file in files {
                    let Ok(file_path) = file else {
                        continue;
                    };
                    if let Ok(file_url) = url_from_file_path(&file_path) {
                        let Ok(content) = tokio::fs::read_to_string(&file_path).await else {
                            continue;
                        };
                        let version = {
                            backend
                                .document_sources
                                .write()
                                .await
                                .entry(file_url.clone())
                                .or_insert_with(|| DocumentSource::new(content, None))
                                .version
                        };

                        total_diagnostic_targets.push(WorkspaceDiagnosticTarget {
                            uri: file_url,
                            version,
                        });
                    }
                }
            }
        }
    }

    if total_diagnostic_targets.is_empty() {
        None
    } else {
        Some(total_diagnostic_targets)
    }
}

struct DiagnosticsResult {
    diagnostics: Vec<tower_lsp::lsp_types::Diagnostic>,
    version: Option<i32>,
}

async fn get_diagnostics_result(
    backend: &Backend,
    text_document_uri: &Url,
) -> Option<DiagnosticsResult> {
    let ConfigSchemaStore {
        config,
        schema_store,
    } = backend
        .config_manager
        .config_schema_store_for_url(text_document_uri)
        .await;

    if !config
        .lsp()
        .and_then(|server| server.diagnostics.as_ref())
        .and_then(|diagnostics| diagnostics.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.diagnostics.enabled` is false");
        return None;
    }

    let root = backend.get_incomplete_ast(text_document_uri).await?;

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(text_document_uri)))
        .await
        .ok()
        .flatten();

    let document_tombi_comment_directive =
        tombi_comment_directive::get_document_tombi_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(
            document_tombi_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let document_sources = backend.document_sources.read().await;

    match document_sources.get(text_document_uri) {
        Some(document) => Some(DiagnosticsResult {
            diagnostics: match tombi_linter::Linter::new(
                toml_version,
                config.lint.as_ref().unwrap_or(&LintOptions::default()),
                Some(Either::Left(text_document_uri)),
                &schema_store,
            )
            .lint(&document.text)
            .await
            {
                Ok(_) => Vec::with_capacity(0),
                Err(diagnostics) => diagnostics.into_iter().unique().map(Into::into).collect(),
            },
            version: document.version,
        }),
        None => None,
    }
}
