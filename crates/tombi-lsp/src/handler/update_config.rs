use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::backend::Backend;

pub async fn handle_update_config(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    log::info!("handle_update_config");
    log::trace!("{:?}", params);

    let text_document_uri: tombi_uri::Uri = params.uri.into();

    if let Ok(config_path) = text_document_uri.to_file_path() {
        match serde_tombi::config::try_from_path(&config_path) {
            Ok(Some(config)) => {
                match backend
                    .config_manager
                    .update_config_with_path(config, &config_path)
                    .await
                {
                    Ok(_) => {
                        log::info!("Updated config: {}", text_document_uri);
                        return Ok(true);
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to update config for {config_path}: {err}",
                            config_path = config_path.display()
                        );
                    }
                }
            }
            Ok(None) => {}
            Err(err) => {
                log::error!(
                    "Failed to load config for update for {config_path}: {err}",
                    config_path = config_path.display()
                );
            }
        }
    }

    Ok(false)
}
