use std::sync::Arc;

use bytes::Bytes;
use tombi_future::Boxable;

use super::{ContentValidator, FetchError, HttpClient, HttpFuture, Options};
use crate::schema_fetch_policy::SchemaFetchPolicy;

/// An [`HttpClient`] that serves responses through the on-disk cache shared by schemas and configs.
///
/// The access policy is checked before the cache is consulted, so revoking trust for a host
/// also stops serving what was cached from it.
///
/// Order: fresh cache, then (offline) stale cache, then fetch, then stale cache when the
/// fetch fails. A refusal by the access policy is terminal and never falls back to the cache.
#[derive(Debug, Clone)]
pub struct CachedHttpClient {
    inner: Arc<dyn HttpClient>,
    options: Options,
    policy: Arc<SchemaFetchPolicy>,
}

impl CachedHttpClient {
    pub fn new(inner: Arc<dyn HttpClient>, options: Options) -> Self {
        let policy = Arc::new(SchemaFetchPolicy::from_options(&crate::Options::default()));
        Self::with_policy(inner, options, policy)
    }

    pub(crate) fn with_policy(
        inner: Arc<dyn HttpClient>,
        options: Options,
        policy: Arc<SchemaFetchPolicy>,
    ) -> Self {
        Self {
            inner,
            options,
            policy,
        }
    }

    /// Like [`HttpClient::get_bytes`], but `is_valid` tells whether the content is usable.
    ///
    /// Unusable content is neither persisted nor served from the cache.
    pub async fn get_validated(
        &self,
        url: &str,
        is_valid: &ContentValidator<'_>,
    ) -> Result<Bytes, FetchError> {
        let cache_path = match url.parse::<tombi_uri::Uri>() {
            Ok(uri) => {
                if matches!(uri.scheme(), "http" | "https") {
                    self.policy.check_http_url(&uri).map_err(FetchError::from)?;
                }
                tombi_cache::get_cache_file_path(&cache_key_uri(&uri)).await
            }
            Err(_) => None,
        };

        if let Ok(Some(content)) =
            tombi_cache::read_from_cache(cache_path.as_deref(), self.options.cache.as_ref()).await
            && is_valid(content.as_bytes())
        {
            log::trace!("load from cache: {url}");
            return Ok(Bytes::from(content));
        }

        if self.options.offline {
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
            Err(error) if error.is_policy_denied() => Err(error),
            Err(error) => match self.read_stale_cache(cache_path.as_deref(), is_valid).await {
                Some(bytes) => Ok(bytes),
                None => Err(error),
            },
        }
    }

    /// Whether the server answers 404 or 410 for `url`.
    ///
    /// Unlike [`Self::get_validated`], a stale cache entry does not hide the answer and
    /// nothing is cached. A fresh cache entry shows that the file exists, so it is
    /// answered without a request. Failures other than 404 and 410, offline mode and a
    /// refusal by the access policy are no evidence that the file is missing.
    pub async fn is_missing(&self, url: &str) -> bool {
        let Ok(uri) = url.parse::<tombi_uri::Uri>() else {
            return false;
        };
        if matches!(uri.scheme(), "http" | "https") && self.policy.check_http_url(&uri).is_err() {
            return false;
        }
        if self.options.offline {
            return false;
        }

        let cache_path = tombi_cache::get_cache_file_path(&cache_key_uri(&uri)).await;
        if let Ok(Some(_)) =
            tombi_cache::read_from_cache(cache_path.as_deref(), self.options.cache.as_ref()).await
        {
            return false;
        }

        matches!(
            self.inner.get_bytes(url).await,
            Err(FetchError::StatusNotOk { status: 404 | 410 })
        )
    }

    async fn read_stale_cache(
        &self,
        cache_path: Option<&std::path::Path>,
        is_valid: &ContentValidator<'_>,
    ) -> Option<Bytes> {
        let mut cache_options = self.options.cache.clone().unwrap_or_default();
        cache_options.cache_ttl = None;
        tombi_cache::read_from_cache(cache_path, Some(&cache_options))
            .await
            .ok()
            .flatten()
            .filter(|content| is_valid(content.as_bytes()))
            .map(Bytes::from)
    }
}

/// The cache path ignores the query and port of a URL, so fold them into an extra path
/// segment to keep `?revision=one` and `?revision=two` apart.
fn cache_key_uri(uri: &tombi_uri::Uri) -> tombi_uri::Uri {
    use sha2::{Digest, Sha256};

    if uri.query().is_none() && uri.port().is_none() {
        return uri.clone();
    }

    let mut key = uri.clone();
    let discriminator = format!(
        "{}?{}",
        uri.port().map(|port| port.to_string()).unwrap_or_default(),
        uri.query().unwrap_or_default()
    );
    key.set_query(None);
    key.set_port(None).ok();
    key.set_fragment(None);
    let digest = Sha256::digest(discriminator.as_bytes());
    let suffix: String = digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if let Ok(mut segments) = key.path_segments_mut() {
        segments.pop_if_empty().push(&format!("__{suffix}"));
    }
    key
}

impl HttpClient for CachedHttpClient {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        self.get_validated(url, &|_| true).boxed()
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
        write_cache_for(URL, content, age).await;
    }

    async fn write_cache_for(url: &str, content: &[u8], age: std::time::Duration) {
        let uri = url.parse::<tombi_uri::Uri>().unwrap();
        let path = tombi_cache::get_cache_file_path(&cache_key_uri(&uri))
            .await
            .unwrap();
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
            Options {
                offline: false,
                cache: cache_options(std::time::Duration::from_secs(60)),
            },
        );

        assert_eq!(client.get_bytes(URL).await.unwrap(), "fetched");
        assert_eq!(client.get_bytes(URL).await.unwrap(), "fetched");
        assert_eq!(inner.calls(), 1);
    }

    #[tokio::test]
    async fn query_and_port_are_part_of_the_cache_key() {
        let _cache_home = TestCacheHome::new();
        let inner = FakeHttpClient::new(Ok("fetched"));
        let client = CachedHttpClient::new(
            inner.clone(),
            Options {
                offline: false,
                cache: cache_options(std::time::Duration::from_secs(60)),
            },
        );

        for url in [
            "https://example.invalid/tombi.toml?revision=one",
            "https://example.invalid/tombi.toml?revision=two",
            "https://example.invalid:8443/tombi.toml?revision=one",
        ] {
            client.get_bytes(url).await.unwrap();
        }
        assert_eq!(inner.calls(), 3);

        client
            .get_bytes("https://example.invalid/tombi.toml?revision=one")
            .await
            .unwrap();
        assert_eq!(inner.calls(), 3);
    }

    #[tokio::test]
    async fn policy_denial_takes_precedence_over_fresh_cache() {
        let _cache_home = TestCacheHome::new();
        let url = "http://127.0.0.1/tombi.toml";
        write_cache_for(url, b"cached", std::time::Duration::ZERO).await;
        let inner = FakeHttpClient::new(Ok("fetched"));
        let client = CachedHttpClient::new(
            inner.clone(),
            Options {
                offline: false,
                cache: cache_options(std::time::Duration::from_secs(60)),
            },
        );

        let error = client.get_bytes(url).await.unwrap_err();
        assert!(error.is_policy_denied());
        assert_eq!(inner.calls(), 0);
    }

    #[tokio::test]
    async fn invalid_content_is_not_cached() {
        let _cache_home = TestCacheHome::new();
        let inner = FakeHttpClient::new(Ok("<html>"));
        let client = CachedHttpClient::new(
            inner.clone(),
            Options {
                offline: false,
                cache: cache_options(std::time::Duration::from_secs(60)),
            },
        );

        let is_valid = |_: &[u8]| false;
        client.get_validated(URL, &is_valid).await.unwrap();
        client.get_validated(URL, &is_valid).await.unwrap();
        assert_eq!(inner.calls(), 2);
    }

    fn online(ttl: std::time::Duration) -> Options {
        Options {
            offline: false,
            cache: cache_options(ttl),
        }
    }

    #[tokio::test]
    async fn stale_cache_does_not_hide_a_missing_file() {
        let _cache_home = TestCacheHome::new();
        write_cache(b"stale", std::time::Duration::from_secs(60 * 60 * 25)).await;
        let inner = FakeHttpClient::new(Err(FetchError::StatusNotOk { status: 404 }));
        let client =
            CachedHttpClient::new(inner.clone(), online(std::time::Duration::from_secs(60)));

        assert!(client.is_missing(URL).await);
    }

    #[tokio::test]
    async fn fresh_cache_shows_that_the_file_exists_without_a_request() {
        let _cache_home = TestCacheHome::new();
        write_cache(b"fresh", std::time::Duration::ZERO).await;
        let inner = FakeHttpClient::new(Err(FetchError::StatusNotOk { status: 404 }));
        let client =
            CachedHttpClient::new(inner.clone(), online(std::time::Duration::from_secs(60)));

        assert!(!client.is_missing(URL).await);
        assert_eq!(inner.calls(), 0);
    }

    #[tokio::test]
    async fn only_404_and_410_mean_missing() {
        let _cache_home = TestCacheHome::new();
        for (response, missing) in [
            (FetchError::StatusNotOk { status: 410 }, true),
            (FetchError::StatusNotOk { status: 500 }, false),
            (
                FetchError::FetchFailed {
                    reason: "unreachable".to_string(),
                },
                false,
            ),
        ] {
            let client = CachedHttpClient::new(
                FakeHttpClient::new(Err(response)),
                online(std::time::Duration::from_secs(60)),
            );
            assert_eq!(client.is_missing(URL).await, missing);
        }
    }

    #[tokio::test]
    async fn offline_never_means_missing() {
        let _cache_home = TestCacheHome::new();
        let inner = FakeHttpClient::new(Err(FetchError::StatusNotOk { status: 404 }));
        let client = CachedHttpClient::new(
            inner.clone(),
            Options {
                offline: true,
                cache: cache_options(std::time::Duration::from_secs(60)),
            },
        );

        assert!(!client.is_missing(URL).await);
        assert_eq!(inner.calls(), 0);
    }

    #[tokio::test]
    async fn offline_serves_stale_cache_without_fetching() {
        let _cache_home = TestCacheHome::new();
        write_cache(b"stale", std::time::Duration::from_secs(60 * 60 * 25)).await;
        let inner = FakeHttpClient::new(Ok("fetched"));
        let client = CachedHttpClient::new(
            inner.clone(),
            Options {
                offline: true,
                cache: None,
            },
        );

        assert_eq!(client.get_bytes(URL).await.unwrap(), "stale");
        assert_eq!(inner.calls(), 0);
    }

    #[tokio::test]
    async fn offline_without_cache_fails_with_offline() {
        let _cache_home = TestCacheHome::new();
        let inner = FakeHttpClient::new(Ok("fetched"));
        let client = CachedHttpClient::new(
            inner.clone(),
            Options {
                offline: true,
                cache: None,
            },
        );

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
            Options {
                offline: false,
                cache: cache_options(std::time::Duration::from_secs(60)),
            },
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
            Options {
                offline: false,
                cache: cache_options(std::time::Duration::from_secs(60)),
            },
        );

        assert!(matches!(
            client.get_bytes(URL).await,
            Err(FetchError::HostNotTrusted)
        ));
    }
}
