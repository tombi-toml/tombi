mod error;
pub use error::FetchError;

use bytes::Bytes;
pub type HttpFuture<'a, T> = tombi_future::BoxFuture<'a, T>;

/// HTTP operations required by the schema store.
pub trait HttpClient: std::fmt::Debug + Send + Sync {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>>;
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

#[cfg(feature = "reqwest")]
mod reqwest_client;
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
