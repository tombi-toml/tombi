use tower_lsp_server::gen_lsp_types::TextDocumentIdentifier;

use crate::backend::Backend;

pub async fn handle_update_config(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<bool, tower_lsp_server::jsonrpc::Error> {
    log::info!("handle_update_config");
    log::trace!("{:?}", params);

    let text_document_uri: tombi_uri::Uri = params.uri.into();

    if let Ok(config_path) = text_document_uri.to_file_path() {
        match backend.config_manager.reload_config(&config_path).await {
            Ok(true) => {
                backend.workspace_diagnostics_cache.write().await.reset();
                return Ok(true);
            }
            Ok(false) => {}
            Err(error) => log::error!("failed to update config {config_path:?}: {error}"),
        }
    }

    Ok(false)
}
