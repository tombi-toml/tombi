use tower_lsp::lsp_types::InitializedParams;

use crate::{backend::Backend, handler::diagnostic::publish_workspace_diagnostics};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_initialized(backend: &Backend, params: InitializedParams) {
    tracing::info!("handle_initialized");
    tracing::trace!(?params);

    tracing::info!("Publishing workspace diagnostics...");
    publish_workspace_diagnostics(backend).await;
}
