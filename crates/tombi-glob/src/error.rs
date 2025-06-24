use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid glob pattern: '{pattern}'")]
    InvalidPattern { pattern: String },

    #[error("Empty pattern provided")]
    EmptyPattern,

    #[error("IO error while walking directory '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Search root path does not exist: '{path}'")]
    RootPathNotFound { path: PathBuf },

    #[error("Search root path is not a directory: '{path}'")]
    RootPathNotDirectory { path: PathBuf },

    #[error("Failed to acquire thread synchronization lock")]
    LockError,
}

impl Error {
    pub fn invalid_pattern(pattern: impl Into<String>) -> Self {
        Self::InvalidPattern {
            pattern: pattern.into(),
        }
    }

    pub fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::IoError {
            path: path.into(),
            source,
        }
    }
}
