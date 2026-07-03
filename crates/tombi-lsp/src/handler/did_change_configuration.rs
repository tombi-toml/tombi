use tower_lsp::lsp_types::DidChangeConfigurationParams;

use crate::backend::Backend;

pub async fn handle_did_change_configuration(
    backend: &Backend,
    params: DidChangeConfigurationParams,
) {
    log::info!("handle_did_change_configuration");
    log::trace!("{:?}", params);

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
                log::info!("Updating editor config: {:?}", config);
                backend.config_manager.update_editor_config(config).await;
            }
            Err(err) => {
                log::error!("Failed to parse editor config: {}", err);
            }
        }
    } else {
        log::debug!("No tombi settings found in didChangeConfiguration");
    }
}
