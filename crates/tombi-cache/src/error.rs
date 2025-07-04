use std::path::PathBuf;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("cache file read failed: {cache_file_path}, reason: {reason}")]
    CacheFileReadFailed {
        cache_file_path: PathBuf,
        reason: String,
    },

    #[error("cache file parent directory not found: {cache_file_path}")]
    CacheFileParentDirectoryNotFound { cache_file_path: PathBuf },

    #[error("failed to save to cache: {cache_file_path}, reason: {reason}")]
    CacheFileSaveFailed {
        cache_file_path: PathBuf,
        reason: String,
    },
}

impl Error {
    #[inline]
    pub fn code(&self) -> &'static str {
        match self {
            Self::CacheFileReadFailed { .. } => "cache-file-read-failed",
            Self::CacheFileParentDirectoryNotFound { .. } => {
                "cache-file-parent-directory-not-found"
            }
            Self::CacheFileSaveFailed { .. } => "cache-file-save-failed",
        }
    }
}
