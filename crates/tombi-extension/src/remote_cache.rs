use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::de::DeserializeOwned;
use tombi_cache::{get_cache_file_path, read_from_cache, save_to_cache};
use tombi_schema_store::HttpClient;

const CACHE_INDEX_FILE_NAME: &str = "__index__.json";

pub async fn get_cached_remote_file_path(url: &str) -> Option<PathBuf> {
    let uri = tombi_uri::Uri::from_str(url).ok()?;
    let cache_uri = get_cache_uri_for_remote_url(&uri)?;
    get_cache_file_path(&cache_uri).await
}

pub async fn fetch_cached_remote_json<T: DeserializeOwned>(
    url: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<T> {
    let cache_file_path = get_cached_remote_file_path(url).await;

    fetch_cached_remote_json_from_path(url, cache_file_path.as_deref(), offline, cache_options)
        .await
}

fn get_cache_uri_for_remote_url(remote_uri: &tombi_uri::Uri) -> Option<tombi_uri::Uri> {
    if remote_uri.path().ends_with(".json") {
        return Some(remote_uri.clone());
    }

    let mut cache_uri = remote_uri.clone();
    let mut path_segments = cache_uri.path_segments_mut().ok()?;
    path_segments.push(CACHE_INDEX_FILE_NAME);
    drop(path_segments);
    Some(cache_uri)
}

async fn fetch_cached_remote_json_from_path<T: DeserializeOwned>(
    url: &str,
    cache_file_path: Option<&Path>,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<T> {
    if let Some(cache_file_path) = cache_file_path
        && let Some(cached_value) = load_cached_json(url, cache_file_path, cache_options).await
    {
        return Some(cached_value);
    }

    if offline {
        if let Some(cached_value) =
            load_cached_json_ignoring_ttl(url, cache_file_path, cache_options).await
        {
            return Some(cached_value);
        }
        log::debug!("offline mode, skip fetch remote metadata from url: {url}");
        return None;
    }

    let bytes = match HttpClient::new().get_bytes(url).await {
        Ok(bytes) => {
            log::debug!("fetch remote metadata from url: {url}");
            bytes
        }
        Err(err) => {
            if let Some(cached_value) =
                load_cached_json_ignoring_ttl(url, cache_file_path, cache_options).await
            {
                return Some(cached_value);
            }
            log::warn!("Failed to fetch remote metadata from {url}: {err}");
            return None;
        }
    };

    if let Err(err) = save_to_cache(cache_file_path, &bytes).await {
        log::warn!("{err}");
    }

    parse_json(url, &bytes)
}

async fn load_cached_json<T: DeserializeOwned>(
    url: &str,
    cache_file_path: &Path,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<T> {
    match read_from_cache(Some(cache_file_path), cache_options).await {
        Ok(Some(cached_text)) => {
            log::debug!("load remote metadata from cache: {url}");
            parse_json(url, cached_text.as_bytes())
        }
        Ok(None) => None,
        Err(err) => {
            log::warn!("Failed to read cached remote metadata from {url}: {err}");
            None
        }
    }
}

async fn load_cached_json_ignoring_ttl<T: DeserializeOwned>(
    url: &str,
    cache_file_path: Option<&Path>,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<T> {
    if let Some(cache_file_path) = cache_file_path {
        let mut owned_cache_options = cache_options.cloned().unwrap_or_default();
        owned_cache_options.cache_ttl = None;
        return load_cached_json(url, cache_file_path, Some(&owned_cache_options)).await;
    }

    None
}

fn parse_json<T: DeserializeOwned>(url: &str, bytes: &[u8]) -> Option<T> {
    match serde_json::from_slice(bytes) {
        Ok(value) => Some(value),
        Err(err) => {
            log::warn!("Failed to parse remote metadata response from {url}: {err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsString,
        path::PathBuf,
        str::FromStr,
        sync::{LazyLock, Mutex, MutexGuard},
        time::Duration,
    };

    use super::*;
    use serde::Deserialize;

    static CACHE_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct TestMetadata {
        name: String,
    }

    struct TestCacheHome {
        _guard: MutexGuard<'static, ()>,
        previous: Option<OsString>,
        _temp_dir: tempfile::TempDir,
    }

    impl TestCacheHome {
        fn new() -> Self {
            let guard = CACHE_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
            let temp_dir = tempfile::tempdir().unwrap();
            let previous = std::env::var_os("XDG_CACHE_HOME");
            // SAFETY: Tests serialize access with a process-wide mutex so env mutation
            // remains scoped to one test at a time.
            unsafe {
                std::env::set_var("XDG_CACHE_HOME", temp_dir.path());
            }
            Self {
                _guard: guard,
                previous,
                _temp_dir: temp_dir,
            }
        }
    }

    impl Drop for TestCacheHome {
        fn drop(&mut self) {
            // SAFETY: Tests serialize access with a process-wide mutex so env mutation
            // remains scoped to one test at a time.
            unsafe {
                if let Some(previous) = &self.previous {
                    std::env::set_var("XDG_CACHE_HOME", previous);
                } else {
                    std::env::remove_var("XDG_CACHE_HOME");
                }
            }
        }
    }

    fn temp_cache_path(test_name: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tombi-{test_name}-{unique}.json"))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn non_json_paths_use_index_file_name() {
        let _cache_home = TestCacheHome::new();
        let root_uri = tombi_uri::Uri::from_str("https://crates.io/api/v1/crates/serde").unwrap();
        let nested_uri =
            tombi_uri::Uri::from_str("https://crates.io/api/v1/crates/serde/1.0.0").unwrap();
        let root_cache_uri = get_cache_uri_for_remote_url(&root_uri).unwrap();
        let nested_cache_uri = get_cache_uri_for_remote_url(&nested_uri).unwrap();

        let root_path = get_cache_file_path(&root_cache_uri).await.unwrap();
        let nested_path = get_cache_file_path(&nested_cache_uri).await.unwrap();

        assert_ne!(root_path, nested_path);
        assert_eq!(root_path.file_name().unwrap(), CACHE_INDEX_FILE_NAME);
        assert_eq!(nested_path.file_name().unwrap(), CACHE_INDEX_FILE_NAME);
        assert_eq!(root_path.parent().unwrap().file_name().unwrap(), "serde");
        assert_eq!(nested_path.parent().unwrap().file_name().unwrap(), "1.0.0");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn root_path_uses_index_file_name() {
        let _cache_home = TestCacheHome::new();
        let root_uri = tombi_uri::Uri::from_str("https://example.invalid").unwrap();
        let root_cache_uri = get_cache_uri_for_remote_url(&root_uri).unwrap();

        let root_path = get_cache_file_path(&root_cache_uri).await.unwrap();

        assert_eq!(root_path.file_name().unwrap(), CACHE_INDEX_FILE_NAME);
        assert_eq!(
            root_path.parent().unwrap().file_name().unwrap(),
            "example.invalid"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn json_paths_are_preserved() {
        let _cache_home = TestCacheHome::new();
        let json_uri =
            tombi_uri::Uri::from_str("https://example.invalid/api/schema/catalog.json").unwrap();
        let cache_uri = get_cache_uri_for_remote_url(&json_uri).unwrap();
        let cache_path = get_cache_file_path(&cache_uri).await.unwrap();

        assert_eq!(cache_uri, json_uri);
        assert_eq!(cache_path.file_name().unwrap(), "catalog.json");
        assert_eq!(cache_path.parent().unwrap().file_name().unwrap(), "schema");
    }

    #[tokio::test]
    async fn prefers_cached_remote_json_before_fetch() {
        let cache_path = temp_cache_path("remote-cache");
        let cache_options = tombi_cache::Options {
            no_cache: None,
            cache_ttl: Some(Duration::from_secs(60)),
        };
        std::fs::write(&cache_path, r#"{"name":"serde"}"#).unwrap();

        let cached = fetch_cached_remote_json_from_path::<TestMetadata>(
            "https://example.invalid/metadata.json",
            Some(&cache_path),
            false,
            Some(&cache_options),
        )
        .await;

        assert_eq!(
            cached,
            Some(TestMetadata {
                name: "serde".to_string()
            })
        );

        let _ = std::fs::remove_file(cache_path);
    }

    #[tokio::test]
    async fn uses_cached_remote_json_while_offline() {
        let cache_path = temp_cache_path("remote-cache-offline");
        let cache_options = tombi_cache::Options {
            no_cache: None,
            cache_ttl: Some(Duration::from_secs(60)),
        };
        std::fs::write(&cache_path, r#"{"name":"requests"}"#).unwrap();

        let cached = fetch_cached_remote_json_from_path::<TestMetadata>(
            "https://example.invalid/metadata.json",
            Some(&cache_path),
            true,
            Some(&cache_options),
        )
        .await;

        assert_eq!(
            cached,
            Some(TestMetadata {
                name: "requests".to_string()
            })
        );

        let _ = std::fs::remove_file(cache_path);
    }

    #[tokio::test]
    async fn ignores_ttl_while_offline_without_cache_options() {
        let cache_path = temp_cache_path("remote-cache-offline-default-options");
        std::fs::write(&cache_path, r#"{"name":"cached"}"#).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&cache_path)
            .unwrap()
            .set_modified(std::time::SystemTime::now() - Duration::from_secs(60 * 60 * 25))
            .unwrap();

        let cached = fetch_cached_remote_json_from_path::<TestMetadata>(
            "https://example.invalid/metadata.json",
            Some(&cache_path),
            true,
            None,
        )
        .await;

        assert_eq!(
            cached,
            Some(TestMetadata {
                name: "cached".to_string()
            })
        );

        let _ = std::fs::remove_file(cache_path);
    }

    #[tokio::test]
    async fn returns_none_while_offline_without_cache() {
        let cache_path = temp_cache_path("remote-cache-miss");
        let cache_options = tombi_cache::Options {
            no_cache: None,
            cache_ttl: Some(Duration::from_secs(60)),
        };
        let cached = fetch_cached_remote_json_from_path::<TestMetadata>(
            "https://example.invalid/metadata.json",
            Some(&cache_path),
            true,
            Some(&cache_options),
        )
        .await;

        assert_eq!(cached, None);
    }

    #[tokio::test]
    async fn ignores_invalid_cached_json_when_online() {
        let cache_path = temp_cache_path("remote-cache-invalid");
        let cache_options = tombi_cache::Options {
            no_cache: None,
            cache_ttl: Some(Duration::from_secs(60)),
        };
        std::fs::write(&cache_path, "not-json").unwrap();

        let cached = fetch_cached_remote_json_from_path::<TestMetadata>(
            "https://example.invalid/metadata.json",
            Some(&cache_path),
            false,
            Some(&cache_options),
        )
        .await;

        assert_eq!(cached, None);

        let _ = std::fs::remove_file(cache_path);
    }
}
