use ahash::AHashSet;
use tombi_glob::search_pattern_matched_paths;

use crate::{
    backend::Backend,
    diagnostic::{get_workspace_configs, publish_diagnostics, WorkspaceConfig},
};

pub async fn push_workspace_diagnostics(
    backend: &Backend,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    tracing::info!("push_workspace_diagnostics");

    for target_path in collect_workspace_diagnostic_targets(backend).await {
        publish_diagnostics(backend, target_path.into(), None).await;
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
                targets.insert(uri);
            }
        }
    }

    targets.into_iter().collect()
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
