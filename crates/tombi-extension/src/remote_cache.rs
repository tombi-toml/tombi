use std::{path::Path, str::FromStr};

use tombi_cache::{get_cache_file_path, read_from_cache, save_to_cache};
use tombi_schema_store::HttpClient;

pub async fn fetch_cached_remote_text(
    url: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<String> {
    let cache_file_path = match tombi_uri::Uri::from_str(url) {
        Ok(uri) => get_cache_file_path(&uri).await,
        Err(err) => {
            log::warn!("Failed to create cache key from remote metadata url {url}: {err}");
            None
        }
    };

    fetch_cached_remote_text_from_path(url, cache_file_path.as_deref(), offline, cache_options)
        .await
}

async fn fetch_cached_remote_text_from_path(
    url: &str,
    cache_file_path: Option<&Path>,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<String> {
    if let Some(cache_file_path) = cache_file_path
        && let Some(cached_text) = load_cached_text(url, cache_file_path, cache_options).await
    {
        return Some(cached_text);
    }

    if offline {
        if let Some(cached_text) =
            load_cached_text_ignoring_ttl(url, cache_file_path, cache_options).await
        {
            return Some(cached_text);
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
            if let Some(cached_text) =
                load_cached_text_ignoring_ttl(url, cache_file_path, cache_options).await
            {
                return Some(cached_text);
            }
            log::warn!("Failed to fetch remote metadata from {url}: {err}");
            return None;
        }
    };

    if let Err(err) = save_to_cache(cache_file_path, &bytes).await {
        log::warn!("{err}");
    }

    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => Some(text),
        Err(err) => {
            log::warn!("Failed to decode remote metadata response from {url}: {err}");
            None
        }
    }
}

async fn load_cached_text(
    url: &str,
    cache_file_path: &Path,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<String> {
    match read_from_cache(Some(cache_file_path), cache_options).await {
        Ok(Some(cached_text)) => {
            log::debug!("load remote metadata from cache: {url}");
            Some(cached_text)
        }
        Ok(None) => None,
        Err(err) => {
            log::warn!("Failed to read cached remote metadata from {url}: {err}");
            None
        }
    }
}

async fn load_cached_text_ignoring_ttl(
    url: &str,
    cache_file_path: Option<&Path>,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<String> {
    if let Some(cache_file_path) = cache_file_path {
        let mut cache_options = cache_options.cloned();
        if let Some(options) = &mut cache_options {
            options.cache_ttl = None;
        }
        return load_cached_text(url, cache_file_path, cache_options.as_ref()).await;
    }

    None
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use super::*;

    fn temp_cache_path(test_name: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tombi-{test_name}-{unique}.json"))
    }

    #[tokio::test]
    async fn prefers_cached_remote_text_before_fetch() {
        let cache_path = temp_cache_path("remote-cache");
        let cache_options = tombi_cache::Options {
            no_cache: None,
            cache_ttl: Some(Duration::from_secs(60)),
        };
        std::fs::write(&cache_path, r#"{"name":"serde"}"#).unwrap();

        let cached = fetch_cached_remote_text_from_path(
            "https://example.invalid/metadata.json",
            Some(&cache_path),
            false,
            Some(&cache_options),
        )
        .await;

        assert_eq!(cached.as_deref(), Some(r#"{"name":"serde"}"#));

        let _ = std::fs::remove_file(cache_path);
    }

    #[tokio::test]
    async fn uses_cached_remote_text_while_offline() {
        let cache_path = temp_cache_path("remote-cache-offline");
        let cache_options = tombi_cache::Options {
            no_cache: None,
            cache_ttl: Some(Duration::from_secs(60)),
        };
        std::fs::write(&cache_path, r#"{"name":"requests"}"#).unwrap();

        let cached = fetch_cached_remote_text_from_path(
            "https://example.invalid/metadata.json",
            Some(&cache_path),
            true,
            Some(&cache_options),
        )
        .await;

        assert_eq!(cached.as_deref(), Some(r#"{"name":"requests"}"#));

        let _ = std::fs::remove_file(cache_path);
    }

    #[tokio::test]
    async fn returns_none_while_offline_without_cache() {
        let cache_path = temp_cache_path("remote-cache-miss");
        let cache_options = tombi_cache::Options {
            no_cache: None,
            cache_ttl: Some(Duration::from_secs(60)),
        };
        let cached = fetch_cached_remote_text_from_path(
            "https://example.invalid/metadata.json",
            Some(&cache_path),
            true,
            Some(&cache_options),
        )
        .await;

        assert_eq!(cached, None);
    }
}
