use crate::Backend;

pub async fn handle_shutdown(backend: &Backend) -> Result<(), tower_lsp::jsonrpc::Error> {
    log::info!("handle_shutdown");
    backend.abort_background_tasks();

    Ok(())
}
