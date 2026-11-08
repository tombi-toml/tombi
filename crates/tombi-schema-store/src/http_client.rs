mod cached;
mod error;
pub use cached::CachedHttpClient;
pub use error::FetchError;

use bytes::Bytes;
pub type HttpFuture<'a, T> = tombi_future::BoxFuture<'a, T>;

/// HTTP operations required by the schema store.
pub trait HttpClient: std::fmt::Debug + Send + Sync {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>>;
}

pub type ContentValidator<'a> = dyn Fn(&[u8]) -> bool + Send + Sync + 'a;

/// Options of the [`HttpClient`] that reads remote files.
#[derive(Debug, Clone)]
pub struct Options {
    /// Serve cached content only, without reaching the network.
    pub offline: bool,
    pub cache: Option<tombi_cache::Options>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            offline: false,
            cache: Some(tombi_cache::Options::default()),
        }
    }
}

impl From<&crate::Options> for Options {
    fn from(options: &crate::Options) -> Self {
        Self {
            offline: options.offline.unwrap_or_default(),
            cache: options.cache.clone(),
        }
    }
}

/// The [`HttpClient`] used to read remote files, backed by the shared on-disk cache.
pub fn http_client(options: Options) -> std::sync::Arc<CachedHttpClient> {
    std::sync::Arc::new(CachedHttpClient::new(
        std::sync::Arc::new(DefaultHttpClient::new()),
        options,
    ))
}

#[allow(dead_code)]
#[inline]
fn http_timeout_secs() -> u64 {
    const DEFAULT_HTTP_TIMEOUT: u64 = 5;

    std::env::var("TOMBI_HTTP_TIMEOUT")
        .ok()
        .or_else(|| std::env::var("HTTP_TIMEOUT").ok())
        .and_then(|timeout| timeout.parse().ok())
        .unwrap_or(DEFAULT_HTTP_TIMEOUT)
}

#[cfg(all(feature = "reqwest", not(target_arch = "wasm32")))]
mod github_credentials;
#[cfg(feature = "reqwest")]
mod reqwest_client;
#[cfg(all(feature = "reqwest", not(target_arch = "wasm32")))]
mod system_proxy;
#[cfg(feature = "reqwest")]
pub use reqwest_client::ReqwestHttpClient;

#[cfg(all(feature = "gloo-net", target_arch = "wasm32"))]
mod gloo_net_client;
#[cfg(all(feature = "gloo-net", target_arch = "wasm32"))]
pub use gloo_net_client::GlooNetHttpClient;

#[cfg(all(
    feature = "gloo-net",
    not(target_arch = "wasm32"),
    not(feature = "reqwest")
))]
compile_error!("the gloo-net HTTP client is only available on wasm32 targets");

#[cfg(all(
    feature = "reqwest",
    not(all(target_arch = "wasm32", feature = "gloo-net"))
))]
pub type DefaultHttpClient = ReqwestHttpClient;

#[cfg(all(feature = "gloo-net", target_arch = "wasm32"))]
pub type DefaultHttpClient = GlooNetHttpClient;

// Provide a stub when no built-in client feature is enabled. This keeps the
// trait usable for callers that inject their own client.
#[cfg(not(any(feature = "reqwest", all(feature = "gloo-net", target_arch = "wasm32"))))]
#[derive(Debug, Clone)]
pub struct DefaultHttpClient;

#[cfg(not(any(feature = "reqwest", all(feature = "gloo-net", target_arch = "wasm32"))))]
impl Default for DefaultHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(any(feature = "reqwest", all(feature = "gloo-net", target_arch = "wasm32"))))]
impl DefaultHttpClient {
    pub fn new() -> Self {
        Self
    }

    /// The stub cannot connect anywhere, so the policy is not needed.
    pub(crate) fn with_policy(
        _policy: std::sync::Arc<crate::schema_fetch_policy::SchemaFetchPolicy>,
    ) -> Self {
        Self
    }
}

#[cfg(not(any(feature = "reqwest", all(feature = "gloo-net", target_arch = "wasm32"))))]
impl HttpClient for DefaultHttpClient {
    fn get_bytes<'a>(&'a self, _url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        use tombi_future::Boxable;

        async {
            Err(FetchError::FetchFailed {
                reason: "No HTTP client feature enabled".to_string(),
            })
        }
        .boxed()
    }
}
