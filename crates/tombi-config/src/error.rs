#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unsupported config uri: {config_uri}")]
    ConfigUriUnsupported { config_uri: tombi_uri::Uri },

    #[error("failed to parse config uri: {config_uri}")]
    ConfigUriParseFailed { config_uri: tombi_uri::Uri },

    #[error("config file not found: {config_path:?}")]
    ConfigFileNotFound { config_path: std::path::PathBuf },

    #[error("failed to read {config_path:?}: {reason}")]
    ConfigFileReadFailed {
        config_path: std::path::PathBuf,
        reason: String,
    },

    #[error("failed to parse {config_path:?}: {reason}")]
    ConfigFileParseFailed {
        config_path: std::path::PathBuf,
        reason: String,
    },

    #[error("unsupported config file: {config_path:?}")]
    ConfigFileUnsupported { config_path: std::path::PathBuf },
}
