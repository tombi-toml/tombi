use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_update_config(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_update_config");
    tracing::trace!(?params);

    if let Ok(params_path) = tombi_url::url_to_file_path(&params.uri) {
        if let Ok((config, config_path)) = serde_tombi::config::load_with_path() {
            if let Some(config_path) = config_path {
                if config_path == params_path {
                    backend.set_config(&params_path, config).await;
                } else {
                    tracing::info!(
                        "Not used as a Tombi Language Server config file: {}",
                        params.uri
                    );
                }
            } else {
                tracing::info!(
                    "Use default config, skip Tombi Language Server config update: {}",
                    params.uri
                );
            }
        } else {
            tracing::error!("Tombi config load failed: {}", params.uri);
        }
    }

    Ok(false)
}
