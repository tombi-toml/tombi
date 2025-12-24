use std::path::PathBuf;

use tombi_config::{Config, ConfigLevel};

/// Load configuration from the specified path or use default loading behavior.
pub fn load_config(
    config_path: Option<PathBuf>,
) -> Result<(Config, Option<PathBuf>, ConfigLevel), Box<dyn std::error::Error>> {
    if let Some(config_path) = config_path {
        let config_path =
            config_path
                .canonicalize()
                .map_err(|_| tombi_config::Error::ConfigFileNotFound {
                    config_path: config_path.clone(),
                })?;
        match serde_tombi::config::try_from_path(&config_path) {
            Ok(Some(config)) => Ok((config, Some(config_path), ConfigLevel::Project)),
            Ok(None) => {
                tracing::warn!("no [tool.tombi] found in {}", config_path.display());
                Ok((Config::default(), None, ConfigLevel::Default))
            }
            Err(error) => Err(Box::new(error)),
        }
    } else {
        // Use default config loading behavior
        serde_tombi::config::load_with_path_and_level(std::env::current_dir().ok())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
