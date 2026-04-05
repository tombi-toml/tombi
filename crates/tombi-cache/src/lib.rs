mod error;
mod options;
pub use error::Error;
pub use options::{DEFAULT_CACHE_TTL, Options};
pub const CACHE_INDEX_FILE_NAME: &str = "__index__.json";

pub async fn get_tombi_cache_dir_path() -> Option<std::path::PathBuf> {
    if let Ok(tombi_cache_home) = std::env::var("TOMBI_CACHE_HOME") {
        return ensure_cache_dir(std::path::PathBuf::from(tombi_cache_home)).await;
    }

    if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
        let mut cache_dir_path = std::path::PathBuf::from(xdg_cache_home);
        cache_dir_path.push("tombi");

        return ensure_cache_dir(cache_dir_path).await;
    }

    if let Some(home_dir) = dirs::home_dir() {
        let mut cache_dir_path = home_dir;
        cache_dir_path.push(".cache");
        cache_dir_path.push("tombi");
        return ensure_cache_dir(cache_dir_path).await;
    }

    None
}

pub async fn get_cache_file_path(cache_file_uri: &tombi_uri::Uri) -> Option<std::path::PathBuf> {
    get_tombi_cache_dir_path().await.map(|mut dir_path| {
        dir_path.push(cache_file_uri.scheme());
        if let Some(host) = cache_file_uri.host() {
            dir_path.push(host.to_string());
        }
        if let Some(path_segments) = cache_file_uri.path_segments() {
            for segment in path_segments {
                dir_path.push(segment)
            }
        }
        if matches!(cache_file_uri.scheme(), "http" | "https")
            && !cache_file_uri.path().ends_with(".json")
        {
            dir_path.push(CACHE_INDEX_FILE_NAME);
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

    if let Some(cache_file_path) = cache_file_path
        && cache_file_path.is_file()
    {
        let cache_ttl = options
            .map(|opts| opts.cache_ttl)
            .unwrap_or_else(|| Options::default().cache_ttl);
        if let Some(ttl) = cache_ttl {
            let Ok(metadata) = tokio::fs::metadata(cache_file_path).await else {
                return Ok(None);
            };
            if let Ok(modified) = metadata.modified()
                && let Ok(elapsed) = modified.elapsed()
                && elapsed > ttl
            {
                return Ok(None);
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
                if let Ok(file_type) = entry.file_type().await
                    && file_type.is_dir()
                {
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
        return Ok(true);
    }

    Ok(false)
}

async fn ensure_cache_dir(cache_dir_path: std::path::PathBuf) -> Option<std::path::PathBuf> {
    if let Err(error) = tokio::fs::create_dir_all(&cache_dir_path).await {
        log::warn!("Failed to create cache directory: {error}");
        return None;
    }

    Some(cache_dir_path)
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsString,
        str::FromStr,
        sync::{LazyLock, Mutex, MutexGuard},
    };

    use super::*;

    static CACHE_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct TestCacheHome {
        _guard: MutexGuard<'static, ()>,
        previous_tombi: Option<OsString>,
        previous_xdg: Option<OsString>,
        temp_dir: std::path::PathBuf,
    }

    impl TestCacheHome {
        fn new() -> Self {
            Self::with_env(None)
        }

        fn with_env(tombi_cache_home: Option<&std::path::Path>) -> Self {
            let guard = CACHE_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let temp_dir = std::env::temp_dir().join(format!("tombi-cache-test-{unique}"));
            std::fs::create_dir_all(&temp_dir).unwrap();
            let previous_tombi = std::env::var_os("TOMBI_CACHE_HOME");
            let previous_xdg = std::env::var_os("XDG_CACHE_HOME");
            // SAFETY: Tests serialize access with a process-wide mutex so env mutation
            // remains scoped to one test at a time.
            unsafe {
                if let Some(tombi_cache_home) = tombi_cache_home {
                    std::env::set_var("TOMBI_CACHE_HOME", tombi_cache_home);
                } else {
                    std::env::remove_var("TOMBI_CACHE_HOME");
                }
                std::env::set_var("XDG_CACHE_HOME", &temp_dir);
            }
            Self {
                _guard: guard,
                previous_tombi,
                previous_xdg,
                temp_dir,
            }
        }
    }

    impl Drop for TestCacheHome {
        fn drop(&mut self) {
            // SAFETY: Tests serialize access with a process-wide mutex so env mutation
            // remains scoped to one test at a time.
            unsafe {
                if let Some(previous) = &self.previous_tombi {
                    std::env::set_var("TOMBI_CACHE_HOME", previous);
                } else {
                    std::env::remove_var("TOMBI_CACHE_HOME");
                }

                if let Some(previous) = &self.previous_xdg {
                    std::env::set_var("XDG_CACHE_HOME", previous);
                } else {
                    std::env::remove_var("XDG_CACHE_HOME");
                }
            }
            let _ = std::fs::remove_dir_all(&self.temp_dir);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn appends_index_file_to_non_json_http_paths() {
        let _cache_home = TestCacheHome::new();
        let uri = tombi_uri::Uri::from_str("https://crates.io/api/v1/crates/countme").unwrap();

        let cache_path = get_cache_file_path(&uri).await.unwrap();

        assert_eq!(cache_path.file_name().unwrap(), CACHE_INDEX_FILE_NAME);
        assert_eq!(cache_path.parent().unwrap().file_name().unwrap(), "countme");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn preserves_json_http_paths() {
        let _cache_home = TestCacheHome::new();
        let uri =
            tombi_uri::Uri::from_str("https://www.schemastore.org/api/json/catalog.json").unwrap();

        let cache_path = get_cache_file_path(&uri).await.unwrap();

        assert_eq!(cache_path.file_name().unwrap(), "catalog.json");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn preserves_non_http_paths() {
        let _cache_home = TestCacheHome::new();
        let uri = tombi_uri::Uri::from_str("file:///tmp/example.toml").unwrap();

        let cache_path = get_cache_file_path(&uri).await.unwrap();

        assert_eq!(cache_path.file_name().unwrap(), "example.toml");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn prefers_tombi_cache_home_over_xdg_cache_home() {
        let tombi_cache_home = std::env::temp_dir().join(format!(
            "tombi-cache-home-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _cache_home = TestCacheHome::with_env(Some(&tombi_cache_home));

        let cache_path = get_tombi_cache_dir_path().await.unwrap();

        assert_eq!(cache_path, tombi_cache_home);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn falls_back_to_xdg_cache_home_when_tombi_cache_home_is_unset() {
        let cache_home = TestCacheHome::new();

        let cache_path = get_tombi_cache_dir_path().await.unwrap();

        assert_eq!(cache_path, cache_home.temp_dir.join("tombi"));
    }
}
