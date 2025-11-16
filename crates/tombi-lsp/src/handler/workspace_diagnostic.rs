use ahash::AHashSet;
use tombi_glob::search_pattern_matched_paths;

use crate::{
    backend::Backend,
    diagnostic::{
        get_diagnostics_result, get_workspace_configs, DiagnosticsResult, WorkspaceConfig,
    },
    document::DocumentSource,
};

pub async fn push_workspace_diagnostics(
    backend: &Backend,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    tracing::info!("push_workspace_diagnostics");

    for text_document_uri in collect_workspace_diagnostic_targets(backend).await {
        publish_workspace_diagnostics(backend, text_document_uri).await;
    }

    Ok(())
}

async fn collect_workspace_diagnostic_targets(backend: &Backend) -> Vec<tombi_uri::Uri> {
    let Some(configs) = get_workspace_configs(backend).await else {
        return Vec::with_capacity(0);
    };

    let mut targets = AHashSet::new();
    let home_dir = dirs::home_dir();

    for workspace_config in configs.into_values() {
        if !is_workspace_diagnostic_enabled(&workspace_config) {
            tracing::debug!(
                "`lsp.workspace-diagnostic.enabled` is false in {}",
                workspace_config.workspace_folder_path.display()
            );
            continue;
        }

        if let Some(home_dir) = &home_dir {
            if &workspace_config.workspace_folder_path == home_dir {
                tracing::debug!(
                    "Skip diagnostics for workspace folder matching $HOME: {:?}",
                    workspace_config.workspace_folder_path
                );
                continue;
            }
        }

        let files_options = workspace_config.config.files.clone().unwrap_or_default();

        for matched_path in
            search_pattern_matched_paths(workspace_config.workspace_folder_path, files_options)
                .await
        {
            let Ok(path) = matched_path else {
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

async fn publish_workspace_diagnostics(backend: &Backend, text_document_uri: tombi_uri::Uri) {
    let Some(diagnostics_result) = get_diagnostics_result(backend, &text_document_uri).await else {
        return;
    };

    tracing::trace!(?diagnostics_result);

    let DiagnosticsResult {
        diagnostics,
        version,
    } = diagnostics_result;

    if version.is_some() {
        tracing::debug!(
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
        .lsp()
        .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
        .and_then(|workspace_diagnostic| workspace_diagnostic.enabled)
        .unwrap_or_default()
        .value()
}

pub async fn upsert_document_source(backend: &Backend, text_document_uri: tombi_uri::Uri) -> bool {
    let text_document_path = match text_document_uri.to_file_path() {
        Ok(text_document_path) => text_document_path,
        Err(_) => {
            tracing::warn!("Watcher event for non-file URI: {text_document_uri}");
            return false;
        }
    };

    let Ok(content) = tokio::fs::read_to_string(&text_document_path).await else {
        tracing::warn!(
            "Failed to read file for diagnostics: {:?}",
            text_document_path
        );
        return false;
    };

    let toml_version = backend
        .text_document_toml_version(&text_document_uri, &content)
        .await;

    let mut document_sources = backend.document_sources.write().await;
    if let Some(source) = document_sources.get_mut(&text_document_uri) {
        if source.version.is_some() {
            tracing::debug!("Skip diagnostics for open document: {text_document_uri}");
            return false;
        }

        source.set_text(content, toml_version);
    } else {
        document_sources.insert(
            text_document_uri.clone(),
            DocumentSource::new(
                content,
                None,
                toml_version,
                backend.capabilities.read().await.encoding_kind,
            ),
        );
    }

    true
}
