#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_shutdown(
    backend: &crate::backend::Backend,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_shutdown");

    // Clean up workspace diagnostic state (file watcher, mtime tracker, throttle, dirty files)
    backend
        .workspace_diagnostic_state
        .read()
        .await
        .clear()
        .await;

    Ok(())
}
