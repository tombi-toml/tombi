use tower_lsp_server::ls_types::lsp::DidChangeConfigurationParams;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_did_change_configuration(params: DidChangeConfigurationParams) {
    tracing::info!("handle_did_change_configuration");
    tracing::trace!(?params);
}
