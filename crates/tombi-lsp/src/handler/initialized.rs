use serde_json::json;
use tower_lsp::lsp_types::{
    DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, GlobPattern, InitializedParams,
    Registration, WatchKind,
};

use crate::{backend::Backend, handler::push_workspace_diagnostics};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_initialized(backend: &Backend, params: InitializedParams) {
    tracing::info!("handle_initialized");
    tracing::trace!(?params);

    tracing::info!("Pushing workspace diagnostics...");
    if let Err(error) = push_workspace_diagnostics(backend).await {
        tracing::warn!("Failed to push workspace diagnostics: {error}");
    }

    tracing::info!("Registering workspace TOML watchers...");
    if let Err(error) = register_workspace_toml_watcher(backend).await {
        tracing::warn!("Failed to register TOML file watchers: {error}");
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
