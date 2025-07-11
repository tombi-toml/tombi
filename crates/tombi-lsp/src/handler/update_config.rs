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
                    match backend
                        .schema_store
                        .reload_config(&config, Some(&config_path))
                        .await
                    {
                        Ok(_) => {
                            backend.update_config_with_path(config, config_path).await;
                            tracing::info!("updated config: {}", params.uri);
                            return Ok(true);
                        }
                        Err(err) => {
                            tracing::error!("schema store reload failed: {}", err);
                        }
                    }
                } else {
                    tracing::info!("not used as a config file, update skipped: {}", params.uri);
                }
            } else {
                tracing::info!("use default config, update skipped: {}", params.uri);
            }
        } else {
            tracing::error!("config load failed: {}", params.uri);
        }
    }

    Ok(false)
}
