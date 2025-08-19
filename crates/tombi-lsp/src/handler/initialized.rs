use tower_lsp_server::ls_types::lsp::InitializedParams;

use crate::{backend::Backend, handler::push_workspace_diagnostics};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_initialized(backend: &Backend, params: InitializedParams) {
    tracing::info!("handle_initialized");
    tracing::trace!(?params);

    tracing::info!("Pushing workspace diagnostics...");
    push_workspace_diagnostics(backend).await;
}
