use tower_lsp::lsp_types::DidChangeConfigurationParams;

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_did_change_configuration(
    backend: &Backend,
    params: DidChangeConfigurationParams,
) {
    tracing::info!("handle_did_change_configuration");
    tracing::trace!(?params);

    // Extract tombi settings from the settings object
    // The settings structure is expected to be:
    // {
    //   "tombi": {
    //     "toml-version": "v1.0.0",
    //     ...
    //   }
    // }
    let settings = params.settings;
    if let Some(tombi_settings) = settings.get("tombi") {
        match serde_json::from_value::<tombi_config::Config>(tombi_settings.clone()) {
            Ok(config) => {
                tracing::info!("Updating editor config: {:?}", config);
                backend.config_manager.update_editor_config(config).await;
            }
            Err(err) => {
                tracing::error!("Failed to parse editor config: {}", err);
            }
        }
    } else {
        tracing::debug!("No tombi settings found in didChangeConfiguration");
    }
}
