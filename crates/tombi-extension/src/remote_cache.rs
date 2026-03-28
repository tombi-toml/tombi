use std::{
    fmt::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::de::DeserializeOwned;
use tombi_cache::{get_tombi_cache_dir_path, read_from_cache, save_to_cache};
use tombi_schema_store::HttpClient;

const CACHE_INDEX_FILE_NAME: &str = "__index__.json";

pub async fn fetch_cached_remote_json<T: DeserializeOwned>(
    url: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<T> {
    let cache_file_path = get_cached_remote_json_file_path(url).await;

    fetch_cached_remote_json_from_path(url, cache_file_path.as_deref(), offline, cache_options)
        .await
}

async fn get_cached_remote_json_file_path(url: &str) -> Option<PathBuf> {
    let remote_uri = match tombi_uri::Uri::from_str(url) {
        Ok(uri) => uri,
        Err(err) => {
            log::warn!("Invalid URL for remote cache {url}: {err}");
            return None;
        }
    };

    let mut cache_dir_path = get_tombi_cache_dir_path().await?;
    cache_dir_path.push(sanitize_cache_segment(remote_uri.scheme()));

    let Some(host) = remote_uri.host_str() else {
        log::warn!("Remote cache URL has no host: {url}");
        return None;
    };
    let host_segment = if let Some(port) = remote_uri.port() {
        format!("{}__port_{}", sanitize_cache_segment(host), port)
    } else {
        sanitize_cache_segment(host)
    };
    cache_dir_path.push(host_segment);

    let mut path_segments = remote_uri
        .path_segments()
        .into_iter()
        .flatten()
        .filter(|segment| !segment.is_empty())
        .map(sanitize_cache_segment)
        .collect::<Vec<_>>();
    let file_name = if let Some(last_segment) = path_segments.last().cloned() {
        if last_segment.ends_with(".json") {
            path_segments.pop();
            augment_json_file_name(&last_segment, remote_uri.query(), remote_uri.fragment())
        } else {
            if let Some(query) = remote_uri.query() {
                path_segments.push(format!("__query_{}", encode_cache_suffix(query)));
            }
            if let Some(fragment) = remote_uri.fragment() {
                path_segments.push(format!("__fragment_{}", encode_cache_suffix(fragment)));
            }
            CACHE_INDEX_FILE_NAME.to_string()
        }
    } else {
        if let Some(query) = remote_uri.query() {
            path_segments.push(format!("__query_{}", encode_cache_suffix(query)));
        }
        if let Some(fragment) = remote_uri.fragment() {
            path_segments.push(format!("__fragment_{}", encode_cache_suffix(fragment)));
        }
        CACHE_INDEX_FILE_NAME.to_string()
    };

    for segment in path_segments {
        cache_dir_path.push(segment);
    }
    cache_dir_path.push(file_name);

    Some(cache_dir_path)
}

fn augment_json_file_name(file_name: &str, query: Option<&str>, fragment: Option<&str>) -> String {
    if query.is_none() && fragment.is_none() {
        return file_name.to_string();
    }

    let stem = file_name.strip_suffix(".json").unwrap_or(file_name);
    let mut augmented = stem.to_string();
    if let Some(query) = query {
        augmented.push_str("__query_");
        augmented.push_str(&encode_cache_suffix(query));
    }
    if let Some(fragment) = fragment {
        augmented.push_str("__fragment_");
        augmented.push_str(&encode_cache_suffix(fragment));
    }
    augmented.push_str(".json");
    augmented
}

fn sanitize_cache_segment(segment: &str) -> String {
    if segment.is_empty() || segment == "." || segment == ".." {
        return format!("__segment_{}", encode_cache_suffix(segment));
    }

    let mut sanitized = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                sanitized.push(byte as char)
            }
            _ => {
                sanitized.push('_');
                let _ = write!(sanitized, "{byte:02x}");
            }
        }
    }
    sanitized
}

fn encode_cache_suffix(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 3);
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => encoded.push(byte as char),
            _ => {
                let _ = write!(encoded, "{byte:02x}");
            }
        }
    }
    encoded
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
        let root_path = get_cached_remote_json_file_path("https://crates.io/api/v1/crates/serde")
            .await
            .unwrap();
        let nested_path =
            get_cached_remote_json_file_path("https://crates.io/api/v1/crates/serde/1.0.0")
                .await
                .unwrap();

        assert_ne!(root_path, nested_path);
        assert_eq!(root_path.file_name().unwrap(), CACHE_INDEX_FILE_NAME);
        assert_eq!(nested_path.file_name().unwrap(), CACHE_INDEX_FILE_NAME);
        assert_eq!(root_path.parent().unwrap().file_name().unwrap(), "serde");
        assert_eq!(nested_path.parent().unwrap().file_name().unwrap(), "1.0.0");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn root_path_uses_index_file_name() {
        let _cache_home = TestCacheHome::new();
        let root_path = get_cached_remote_json_file_path("https://example.invalid")
            .await
            .unwrap();

        assert_eq!(root_path.file_name().unwrap(), CACHE_INDEX_FILE_NAME);
        assert_eq!(
            root_path.parent().unwrap().file_name().unwrap(),
            "example.invalid"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn json_paths_are_preserved() {
        let _cache_home = TestCacheHome::new();
        let cache_path =
            get_cached_remote_json_file_path("https://example.invalid/api/schema/catalog.json")
                .await
                .unwrap();
        assert_eq!(cache_path.file_name().unwrap(), "catalog.json");
        assert_eq!(cache_path.parent().unwrap().file_name().unwrap(), "schema");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn query_fragment_and_port_affect_cache_path() {
        let _cache_home = TestCacheHome::new();
        let base =
            get_cached_remote_json_file_path("https://example.invalid/api/schema/catalog.json")
                .await
                .unwrap();
        let with_query = get_cached_remote_json_file_path(
            "https://example.invalid/api/schema/catalog.json?kind=full",
        )
        .await
        .unwrap();
        let with_fragment = get_cached_remote_json_file_path(
            "https://example.invalid/api/schema/catalog.json#section",
        )
        .await
        .unwrap();
        let with_port = get_cached_remote_json_file_path(
            "https://example.invalid:8443/api/schema/catalog.json",
        )
        .await
        .unwrap();

        assert_ne!(base, with_query);
        assert_ne!(base, with_fragment);
        assert_ne!(base, with_port);
        assert!(
            with_query
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("catalog__query_")
        );
        assert!(
            with_fragment
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("catalog__fragment_")
        );
        assert_eq!(
            with_port
                .ancestors()
                .nth(3)
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy(),
            "example.invalid__port_8443"
        );
    }

    #[test]
    fn dangerous_segments_are_sanitized() {
        assert_eq!(sanitize_cache_segment("."), "__segment_2e");
        assert_eq!(sanitize_cache_segment(".."), "__segment_2e2e");
        assert_eq!(sanitize_cache_segment("a/b"), "a_2fb");
        assert_eq!(sanitize_cache_segment("a\\b"), "a_5cb");
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
