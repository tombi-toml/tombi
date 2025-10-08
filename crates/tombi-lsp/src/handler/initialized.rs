use ahash::AHashMap;
use tower_lsp::lsp_types::InitializedParams;

use crate::{
    backend::Backend, file_watcher_manager::FileWatcherConfig, handler::push_workspace_diagnostics,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_initialized(backend: &Backend, params: InitializedParams) {
    tracing::info!("handle_initialized");
    tracing::trace!(?params);

    // Initialize file watcher for workspace diagnostics
    if let Ok(workspace_folders) = backend.client.workspace_folders().await {
        if let Some(folders) = workspace_folders {
            let mut workspace_paths = Vec::new();
            let mut configs = AHashMap::new();

            for folder in folders {
                if let Ok(path) = tombi_uri::Uri::to_file_path(&folder.uri.into()) {
                    workspace_paths.push(path.clone());

                    // Get config for this workspace
                    let config_store = backend
                        .config_manager
                        .config_schema_store_for_file(&path)
                        .await;

                    let watcher_config = FileWatcherConfig {
                        enabled: config_store
                            .config
                            .lsp()
                            .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
                            .and_then(|wd| wd.file_watcher_enabled)
                            .map(|v| v.value())
                            .unwrap_or(true), // Default: enabled
                        debounce_ms: config_store
                            .config
                            .lsp()
                            .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
                            .and_then(|wd| wd.file_watcher_debounce_ms)
                            .unwrap_or(100), // Default: 100ms
                    };

                    configs.insert(path, watcher_config);
                }
            }

            // Initialize file watcher (non-blocking, logs errors internally)
            backend
                .workspace_diagnostic_state
                .write()
                .await
                .initialize_file_watcher(workspace_paths.clone(), configs)
                .await;

            // Mark all files as dirty for initial scan
            if backend
                .workspace_diagnostic_state
                .read()
                .await
                .file_watcher_manager()
                .is_some()
            {
                for workspace_path in workspace_paths {
                    // Get all TOML files in workspace for initial scan
                    if let Some(workspace_path_str) = workspace_path.to_str() {
                        if let Ok((config, config_path, config_level)) =
                            serde_tombi::config::load_with_path_and_level(Some(workspace_path.clone()))
                        {
                            if let tombi_glob::FileSearch::Files(files) =
                                tombi_glob::FileSearch::new(
                                    &[workspace_path_str],
                                    &config,
                                    config_path.as_deref(),
                                    config_level,
                                )
                                .await
                            {
                            let mut file_uris = Vec::new();
                            for file_result in files {
                                if let Ok(file_path) = file_result {
                                    if let Ok(uri) = tombi_uri::Uri::from_file_path(&file_path) {
                                        file_uris.push(uri);
                                    }
                                }
                            }

                                if !file_uris.is_empty() {
                                    tracing::debug!(
                                        "Marking {} files for initial scan in workspace: {:?}",
                                        file_uris.len(),
                                        workspace_path
                                    );
                                    backend
                                        .workspace_diagnostic_state
                                        .read()
                                        .await
                                        .dirty_files_queue()
                                        .mark_initial_scan(workspace_path, file_uris)
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    tracing::info!("Pushing workspace diagnostics...");
    push_workspace_diagnostics(backend).await;
}
