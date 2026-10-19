use std::path::PathBuf;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("cache directory is unavailable")]
    CacheDirectoryUnavailable,

    #[error("unsafe cache directory: {cache_dir_path}")]
    UnsafeCacheDirectory { cache_dir_path: PathBuf },

    #[error("failed to resolve cache directory: {cache_dir_path}, reason: {reason}")]
    CacheDirectoryResolveFailed {
        cache_dir_path: PathBuf,
        reason: String,
    },
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

    #[error("failed to remove cache directory: {cache_dir_path}, reason: {reason}")]
    CacheDirectoryRemoveFailed {
        cache_dir_path: PathBuf,
        reason: String,
    },
}
