use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_update_config(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_update_config");
    tracing::trace!(?params);

    let text_document_uri: tombi_uri::Uri = params.uri.into();

    if let Ok(config_path) = text_document_uri.to_file_path() {
        if let Ok(Some(config)) = serde_tombi::config::try_from_path(&config_path) {
            match backend
                .config_manager
                .update_config_with_path(config, config_path)
                .await
            {
                Ok(_) => {
                    tracing::info!("updated config: {}", text_document_uri);
                    return Ok(true);
                }
                Err(err) => {
                    tracing::error!("failed to update config: {}", err);
                }
            }
        } else {
            tracing::error!("failed to load config for update: {}", text_document_uri);
        }
    }

    Ok(false)
}
