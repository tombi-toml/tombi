use serde_json::json;
use tower_lsp::lsp_types::{
    DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, GlobPattern, InitializedParams,
    MessageType, Registration, WatchKind,
};

use crate::{
    backend::{Backend, DiagnosticMode},
    handler::{push_workspace_diagnostics, workspace_diagnostic::WorkspaceDiagnosticOptions},
};

pub async fn handle_initialized(backend: &Backend, params: InitializedParams) {
    log::info!("handle_initialized");
    log::trace!("{:?}", params);

    let startup_backend = backend.clone();
    let startup_task = tokio::spawn(async move {
        log::info!("Loading config in background...");
        if let Err(error) = startup_backend.config_manager.load().await {
            log::warn!("Failed to load config: {error}");
            startup_backend
                .client
                .show_message(MessageType::WARNING, error.to_string())
                .await;
            return;
        }

        let diagnostic_mode = startup_backend.capabilities.read().await.diagnostic_mode;
        match diagnostic_mode {
            DiagnosticMode::Pull => {
                log::info!("Refreshing pull diagnostics after config load...");
                startup_backend.refresh_pull_diagnostics().await;
            }
            DiagnosticMode::Push => {
                let open_document_uris = startup_backend
                    .document_sources
                    .read()
                    .await
                    .iter()
                    .filter_map(|(uri, source)| source.version.is_some().then_some(uri.clone()))
                    .collect::<Vec<_>>();

                for text_document_uri in open_document_uris {
                    startup_backend.push_diagnostics(text_document_uri).await;
                }

                log::info!("Pushing workspace diagnostics after config load...");
                if let Err(error) = push_workspace_diagnostics(
                    &startup_backend,
                    &WorkspaceDiagnosticOptions {
                        include_open_files: true,
                    },
                )
                .await
                {
                    log::warn!("Failed to push workspace diagnostics: {error}");
                }
            }
        }
    });
    backend.register_background_task(&startup_task);

    log::info!("Registering workspace TOML watchers...");
    if let Err(error) = register_workspace_toml_watcher(backend).await {
        log::warn!("Failed to register TOML file watchers: {error}");
    }
}

async fn register_workspace_toml_watcher(
    backend: &Backend,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    let watcher = FileSystemWatcher {
        glob_pattern: GlobPattern::String("**/*.toml".to_string()),
        kind: Some(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
    };

    let options = DidChangeWatchedFilesRegistrationOptions {
        watchers: vec![watcher],
    };

    let registration = Registration {
        id: "workspace-diagnostics.toml-watcher".to_string(),
        method: "workspace/didChangeWatchedFiles".to_string(),
        register_options: Some(json!(options)),
    };

    backend.client.register_capability(vec![registration]).await
}
