use std::sync::Arc;

use bytes::Bytes;
use tombi_future::Boxable;

use super::{ContentValidator, FetchError, HttpClient, HttpFuture};

/// An [`HttpClient`] that serves responses through the on-disk cache shared by schemas and configs.
///
/// Order: fresh cache, then (offline) stale cache, then fetch, then stale cache when the
/// fetch fails. A refusal by the access policy is terminal and never falls back to the cache.
#[derive(Debug, Clone)]
pub struct CachedHttpClient {
    inner: Arc<dyn HttpClient>,
    cache_options: Option<tombi_cache::Options>,
    offline: bool,
}

impl CachedHttpClient {
    pub fn new(
        inner: Arc<dyn HttpClient>,
        cache_options: Option<tombi_cache::Options>,
        offline: bool,
    ) -> Self {
        Self {
            inner,
            cache_options,
            offline,
        }
    }

    async fn get_bytes_cached(
        &self,
        url: &str,
        is_valid: &ContentValidator<'_>,
    ) -> Result<Bytes, FetchError> {
        let cache_path = match url.parse::<tombi_uri::Uri>() {
            Ok(uri) => tombi_cache::get_cache_file_path(&uri).await,
            Err(_) => None,
        };

        if let Ok(Some(content)) =
            tombi_cache::read_from_cache(cache_path.as_deref(), self.cache_options.as_ref()).await
            && is_valid(content.as_bytes())
        {
            log::trace!("load from cache: {url}");
            return Ok(Bytes::from(content));
        }

        if self.offline {
            return match self.read_stale_cache(cache_path.as_deref(), is_valid).await {
                Some(bytes) => Ok(bytes),
                None => Err(FetchError::Offline),
            };
        }

        match self.inner.get_bytes(url).await {
            Ok(bytes) => {
                if is_valid(&bytes)
                    && let Err(error) =
                        tombi_cache::save_to_cache(cache_path.as_deref(), &bytes).await
                {
                    log::warn!("{error}");
                }
                Ok(bytes)
            }
            Err(error)
                if error.is_policy_denied()
                    || matches!(error, FetchError::AuthenticationFailed { .. }) =>
            {
                Err(error)
            }
            Err(error) => match self.read_stale_cache(cache_path.as_deref(), is_valid).await {
                Some(bytes) => Ok(bytes),
                None => Err(error),
            },
        }
    }

    async fn read_stale_cache(
        &self,
        cache_path: Option<&std::path::Path>,
        is_valid: &ContentValidator<'_>,
    ) -> Option<Bytes> {
        let mut cache_options = self.cache_options.clone().unwrap_or_default();
        cache_options.cache_ttl = None;
        tombi_cache::read_from_cache(cache_path, Some(&cache_options))
            .await
            .ok()
            .flatten()
            .filter(|content| is_valid(content.as_bytes()))
            .map(Bytes::from)
    }
}

impl HttpClient for CachedHttpClient {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        self.get_bytes_cached(url, &|_| true).boxed()
    }

    fn get_validated<'a>(
        &'a self,
        url: &'a str,
        is_valid: &'a ContentValidator<'a>,
    ) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        self.get_bytes_cached(url, is_valid).boxed()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tombi_test_lib::TestCacheHome;

    use super::*;

    const URL: &str = "https://example.invalid/schema.json";

    #[derive(Debug)]
    struct FakeHttpClient {
        response: Result<&'static str, FetchError>,
        calls: AtomicUsize,
    }

    impl FakeHttpClient {
        fn new(response: Result<&'static str, FetchError>) -> Arc<Self> {
            Arc::new(Self {
                response,
                calls: AtomicUsize::new(0),
            })
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl HttpClient for FakeHttpClient {
        fn get_bytes<'a>(&'a self, _url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let response = self
                .response
                .clone()
                .map(|text| Bytes::from_static(text.as_bytes()));
            async move { response }.boxed()
        }
    }

    fn cache_options(ttl: std::time::Duration) -> Option<tombi_cache::Options> {
        Some(tombi_cache::Options {
            no_cache: Some(false),
            cache_ttl: Some(ttl),
        })
    }

    async fn write_cache(content: &[u8], age: std::time::Duration) {
        let uri = URL.parse::<tombi_uri::Uri>().unwrap();
        let path = tombi_cache::get_cache_file_path(&uri).await.unwrap();
        tombi_cache::save_to_cache(Some(&path), content)
            .await
            .unwrap();
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(std::time::SystemTime::now() - age)
            .unwrap();
    }

    #[tokio::test]
    async fn fetched_response_is_cached_and_reused_while_fresh() {
        let _cache_home = TestCacheHome::new();
        let inner = FakeHttpClient::new(Ok("fetched"));
        let client = CachedHttpClient::new(
            inner.clone(),
            cache_options(std::time::Duration::from_secs(60)),
            false,
        );

        assert_eq!(client.get_bytes(URL).await.unwrap(), "fetched");
        assert_eq!(client.get_bytes(URL).await.unwrap(), "fetched");
        assert_eq!(inner.calls(), 1);
    }

    #[tokio::test]
    async fn invalid_content_is_not_cached() {
        let _cache_home = TestCacheHome::new();
        let inner = FakeHttpClient::new(Ok("<html>"));
        let client = CachedHttpClient::new(
            inner.clone(),
            cache_options(std::time::Duration::from_secs(60)),
            false,
        );

        let is_valid = |_: &[u8]| false;
        client.get_validated(URL, &is_valid).await.unwrap();
        client.get_validated(URL, &is_valid).await.unwrap();
        assert_eq!(inner.calls(), 2);
    }

    #[tokio::test]
    async fn offline_serves_stale_cache_without_fetching() {
        let _cache_home = TestCacheHome::new();
        write_cache(b"stale", std::time::Duration::from_secs(60 * 60 * 25)).await;
        let inner = FakeHttpClient::new(Ok("fetched"));
        let client = CachedHttpClient::new(inner.clone(), None, true);

        assert_eq!(client.get_bytes(URL).await.unwrap(), "stale");
        assert_eq!(inner.calls(), 0);
    }

    #[tokio::test]
    async fn offline_without_cache_fails_with_offline() {
        let _cache_home = TestCacheHome::new();
        let inner = FakeHttpClient::new(Ok("fetched"));
        let client = CachedHttpClient::new(inner.clone(), None, true);

        assert!(matches!(
            client.get_bytes(URL).await,
            Err(FetchError::Offline)
        ));
        assert_eq!(inner.calls(), 0);
    }

    #[tokio::test]
    async fn fetch_failure_falls_back_to_stale_cache() {
        let _cache_home = TestCacheHome::new();
        write_cache(b"stale", std::time::Duration::from_secs(120)).await;
        let inner = FakeHttpClient::new(Err(FetchError::StatusNotOk { status: 500 }));
        let client = CachedHttpClient::new(
            inner.clone(),
            cache_options(std::time::Duration::from_secs(60)),
            false,
        );

        assert_eq!(client.get_bytes(URL).await.unwrap(), "stale");
        assert_eq!(inner.calls(), 1);
    }

    #[tokio::test]
    async fn policy_denial_does_not_fall_back_to_cache() {
        let _cache_home = TestCacheHome::new();
        write_cache(b"stale", std::time::Duration::from_secs(120)).await;
        let inner = FakeHttpClient::new(Err(FetchError::HostNotTrusted));
        let client = CachedHttpClient::new(
            inner,
            cache_options(std::time::Duration::from_secs(60)),
            false,
        );

        assert!(matches!(
            client.get_bytes(URL).await,
            Err(FetchError::HostNotTrusted)
        ));
    }
}
