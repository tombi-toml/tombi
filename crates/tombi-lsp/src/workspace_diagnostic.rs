mod cache;

use tombi_glob::search_pattern_matched_paths;

use crate::{
    Backend,
    diagnostic::{DiagnosticsResult, get_diagnostics_result},
    document::DocumentSource,
    workspace_config::get_workspace_configs,
};
pub use cache::WorkspaceDiagnosticsCache;

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

pub async fn collect_workspace_diagnostic_targets(backend: &Backend) -> Vec<tombi_uri::Uri> {
    if let Some(targets) = backend
        .workspace_diagnostics_cache
        .read()
        .await
        .workspace_targets()
    {
        return targets;
    }

    let Some(configs) = get_workspace_configs(backend).await else {
        return Vec::new();
    };

    let mut candidates = tombi_hashmap::HashSet::new();
    let home_dir = dirs::home_dir();

    for workspace_config in configs {
        if !workspace_config.is_workspace_diagnostic_enabled() {
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
                candidates.insert(uri);
            }
        }
    }

    let mut targets = Vec::with_capacity(candidates.len());

    for target in candidates {
        if upsert_document_source(backend, target.clone()).await {
            targets.push(target);
        }
    }

    backend
        .workspace_diagnostics_cache
        .write()
        .await
        .set_workspace_targets(targets.clone().into_iter().collect());

    targets
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

    {
        let mut document_sources = backend.document_sources.write().await;
        if let Some(source) = document_sources.get_mut(&text_document_uri) {
            if source.version.is_some() {
                log::debug!("Skip diagnostics for open document: {text_document_uri}");
                return true;
            }

            source.set_text(content, toml_version);
        } else {
            document_sources.insert(
                text_document_uri.clone(),
                DocumentSource::new(content, None, toml_version, encoding_kind),
            );
        }
    }

    true
}
