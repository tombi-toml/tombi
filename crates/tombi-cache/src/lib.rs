mod error;
mod options;
pub use error::Error;
pub use options::{Options, DEFAULT_CACHE_TTL};

pub async fn get_tombi_cache_dir_path() -> Option<std::path::PathBuf> {
    if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
        let mut cache_dir_path = std::path::PathBuf::from(xdg_cache_home);
        cache_dir_path.push("tombi");

        if !cache_dir_path.is_dir() {
            if let Err(error) = tokio::fs::create_dir_all(&cache_dir_path).await {
                tracing::error!("Failed to create cache directory: {error}");
                return None;
            }
        }
        return Some(cache_dir_path);
    }

    if let Some(home_dir) = dirs::home_dir() {
        let mut cache_dir_path = home_dir.clone();
        cache_dir_path.push(".cache");
        cache_dir_path.push("tombi");
        if !cache_dir_path.is_dir() {
            if let Err(error) = std::fs::create_dir_all(&cache_dir_path) {
                tracing::error!("Failed to create cache directory: {error}");
                return None;
            }
        }
        return Some(cache_dir_path);
    }

    None
}

pub async fn get_cache_file_path(cache_file_url: &url::Url) -> Option<std::path::PathBuf> {
    get_tombi_cache_dir_path().await.map(|mut dir_path| {
        dir_path.push(cache_file_url.scheme());
        if let Some(host) = cache_file_url.host() {
            dir_path.push(host.to_string());
        }
        if let Some(path_segments) = cache_file_url.path_segments() {
            for segment in path_segments {
                dir_path.push(segment)
            }
        }

        dir_path
    })
}

pub async fn read_from_cache(
    cache_file_path: Option<&std::path::Path>,
    options: Option<&Options>,
) -> Result<Option<String>, crate::Error> {
    if options
        .and_then(|options| options.no_cache)
        .unwrap_or_default()
    {
        return Ok(None);
    }

    if let Some(cache_file_path) = cache_file_path {
        if cache_file_path.is_file() {
            if let Some(ttl) = options.and_then(|options| options.cache_ttl) {
                let Ok(metadata) = tokio::fs::metadata(cache_file_path).await else {
                    return Ok(None);
                };
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > ttl {
                            return Ok(None);
                        }
                    }
                }
            }
            return Ok(Some(
                tokio::fs::read_to_string(&cache_file_path)
                    .await
                    .map_err(|err| crate::Error::CacheFileReadFailed {
                        cache_file_path: cache_file_path.to_path_buf(),
                        reason: err.to_string(),
                    })?,
            ));
        }
    }

    Ok(None)
}

pub async fn save_to_cache(
    cache_file_path: Option<&std::path::Path>,
    bytes: &[u8],
) -> Result<(), crate::Error> {
    if let Some(cache_file_path) = cache_file_path {
        if !cache_file_path.is_file() {
            let Some(cache_dir_path) = cache_file_path.parent() else {
                return Err(crate::Error::CacheFileParentDirectoryNotFound {
                    cache_file_path: cache_file_path.to_owned(),
                });
            };

            if let Err(err) = tokio::fs::create_dir_all(cache_dir_path).await {
                return Err(crate::Error::CacheFileSaveFailed {
                    cache_file_path: cache_file_path.to_owned(),
                    reason: err.to_string(),
                });
            }
        }
        if let Err(err) = tokio::fs::write(cache_file_path, &bytes).await {
            return Err(crate::Error::CacheFileSaveFailed {
                cache_file_path: cache_file_path.to_owned(),
                reason: err.to_string(),
            });
        }
    }

    Ok(())
}

pub async fn refresh_cache() -> Result<bool, crate::Error> {
    if let Some(cache_dir_path) = get_tombi_cache_dir_path().await {
        // Remove all contents of the cache directory but keep the directory itself
        if let Ok(mut entries) = tokio::fs::read_dir(&cache_dir_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_dir() {
                        let path = entry.path();
                        if let Err(err) = tokio::fs::remove_dir_all(&path).await {
                            return Err(crate::Error::CacheDirectoryRemoveFailed {
                                cache_dir_path: path,
                                reason: err.to_string(),
                            });
                        }
                    }
                }
            }
        }
        return Ok(true);
    }

    Ok(false)
}
