pub async fn handle_shutdown() -> Result<(), tower_lsp::jsonrpc::Error> {
    log::info!("handle_shutdown");

    Ok(())
}
