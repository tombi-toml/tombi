#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unsupported config uri: {config_uri}")]
    ConfigUriUnsupported { config_uri: tombi_uri::Uri },

    #[error("failed to parse config uri: {config_uri}")]
    ConfigUriParseFailed { config_uri: tombi_uri::Uri },

    #[error("config file not found: {config_path:?}")]
    ConfigFileNotFound { config_path: std::path::PathBuf },

    #[error("failed to read {config_path:?}")]
    ConfigFileReadFailed { config_path: std::path::PathBuf },

    #[error("failed to parse {config_path:?}")]
    ConfigFileParseFailed { config_path: std::path::PathBuf },

    #[error("unsupported config file: {config_path:?}")]
    ConfigFileUnsupported { config_path: std::path::PathBuf },
}
