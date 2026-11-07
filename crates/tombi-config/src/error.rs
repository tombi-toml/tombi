#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unsupported config uri: {config_uri}")]
    ConfigUriUnsupported { config_uri: tombi_uri::Uri },

    #[error("failed to parse config uri: {config_uri}")]
    ConfigUriParseFailed { config_uri: tombi_uri::Uri },

    #[error("config file not found: {config_path:?}")]
    ConfigFileNotFound { config_path: std::path::PathBuf },

    #[error("failed to read config file {config_path:?}")]
    ConfigFileReadFailed { config_path: std::path::PathBuf },

    #[error("failed to parse config file {config_path:?}")]
    ConfigFileParseFailed { config_path: std::path::PathBuf },

    #[error("unsupported config file: {config_path:?}")]
    ConfigFileUnsupported { config_path: std::path::PathBuf },

    #[error("invalid extends reference {reference:?} in {config_source}: {reason}")]
    ConfigExtendsInvalid {
        config_source: std::string::String,
        reference: std::string::String,
        reason: std::string::String,
    },

    #[error("failed to parse inherited config {config_source}: {reason}")]
    ConfigExtendsParseFailed {
        config_source: std::string::String,
        reason: std::string::String,
    },

    #[error("config inheritance cycle: {chain}")]
    ConfigExtendsCycle { chain: std::string::String },

    #[error("config inheritance exceeds the maximum depth of {limit}")]
    ConfigExtendsDepthExceeded { limit: usize },

    #[error("config inheritance exceeds the maximum of {limit} references")]
    ConfigExtendsCountExceeded { limit: usize },

    #[error("failed to fetch config {config_uri}: {reason}")]
    ConfigUriFetchFailed {
        config_uri: tombi_uri::Uri,
        reason: std::string::String,
    },

    #[error("remote config {config_uri} cannot be loaded while offline")]
    ConfigOffline { config_uri: tombi_uri::Uri },

    #[error("config inheritance exceeds the maximum of {limit} schemas and overrides entries")]
    ConfigInheritanceEntriesExceeded { limit: usize },
}
