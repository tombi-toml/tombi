use tombi_config::SUPPORTED_CONFIG_FILENAMES;
use tower_lsp::lsp_types::{
    notification::ShowMessage, MessageType, ShowMessageParams, TextDocumentIdentifier, Url,
};

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_update_config(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_update_config");
    tracing::trace!(?params);

    let TextDocumentIdentifier {
        uri: config_url, ..
    } = params;

    let Some(workspace_folders) = backend.client.workspace_folders().await? else {
        return Ok(false);
    };

    for workspace_folder in workspace_folders {
        let workspace_folder_uri = if !workspace_folder.uri.as_str().ends_with('/') {
            Url::parse(&format!("{}/", workspace_folder.uri.as_str())).unwrap()
        } else {
            workspace_folder.uri
        };

        for config_filename in SUPPORTED_CONFIG_FILENAMES {
            let Ok(workspace_config_url) = workspace_folder_uri.join(config_filename) else {
                continue;
            };

            if config_url == workspace_config_url && update_config(backend, &config_url).await? {
                return Ok(true);
            }
        }
    }

    if let Some(user_config_path) = serde_tombi::config::user_tombi_config_path() {
        if let Ok(user_config_url) = Url::from_file_path(user_config_path) {
            if config_url == user_config_url && update_config(backend, &config_url).await? {
                return Ok(true);
            }
        }
    }

    tracing::info!("No workspace found for config settings: {}", config_url);

    Ok(false)
}

async fn update_config(
    backend: &Backend,
    config_url: &Url,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    match serde_tombi::config::try_from_url(config_url) {
        Ok(Some(config)) => {
            backend.update_workspace_config(config_url, config).await;
            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(err) => {
            backend
                .client
                .send_notification::<ShowMessage>(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: err.to_string(),
                })
                .await;

            Ok(false)
        }
    }
}
