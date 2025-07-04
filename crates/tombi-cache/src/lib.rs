mod error;
pub use error::Error;

fn get_tombi_cache_dir_path() -> Option<std::path::PathBuf> {
    if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
        let mut cache_dir_path = std::path::PathBuf::from(xdg_cache_home);
        cache_dir_path.push("tombi");

        if cache_dir_path.is_dir() {
            return Some(cache_dir_path);
        } else {
            if let Err(error) = std::fs::create_dir_all(&cache_dir_path) {
                tracing::error!("Failed to create cache directory: {error}");
                return None;
            }
        }
    }

    if let Some(home_dir) = dirs::home_dir() {
        let mut cache_dir_path = home_dir.clone();
        cache_dir_path.push(".cache");
        cache_dir_path.push("tombi");
        if cache_dir_path.is_dir() {
            return Some(cache_dir_path);
        } else {
            if let Err(error) = std::fs::create_dir_all(&cache_dir_path) {
                tracing::error!("Failed to create cache directory: {error}");
                return None;
            }
        }
    }

    None
}

pub fn get_cache_file_path(cache_file_url: &url::Url) -> Option<std::path::PathBuf> {
    get_tombi_cache_dir_path().map(|mut dir_path| {
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

pub fn read_from_cache(
    cache_file_path: Option<&std::path::Path>,
    ttl: Option<std::time::Duration>,
) -> Result<Option<String>, crate::Error> {
    if let Some(cache_file_path) = cache_file_path {
        if cache_file_path.is_file() {
            if let Some(ttl) = ttl {
                let Ok(metadata) = std::fs::metadata(cache_file_path) else {
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
            return Ok(Some(std::fs::read_to_string(&cache_file_path).map_err(
                |err| crate::Error::CacheFileReadFailed {
                    cache_file_path: cache_file_path.to_path_buf(),
                    reason: err.to_string(),
                },
            )?));
        }
    }

    Ok(None)
}

pub fn save_to_cache(
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

            if let Err(err) = std::fs::create_dir_all(cache_dir_path) {
                return Err(crate::Error::CacheFileSaveFailed {
                    cache_file_path: cache_file_path.to_owned(),
                    reason: err.to_string(),
                });
            }
        }
        if let Err(err) = std::fs::write(cache_file_path, &bytes) {
            return Err(crate::Error::CacheFileSaveFailed {
                cache_file_path: cache_file_path.to_owned(),
                reason: err.to_string(),
            });
        }
    }

    Ok(())
}
