mod error;

#[cfg(feature = "reqwest01")]
mod reqwest_client;
#[cfg(feature = "reqwest01")]
pub use reqwest_client::HttpClient;

#[cfg(feature = "gloo-net06")]
#[allow(dead_code)]
mod gloo_net_client;
#[cfg(all(feature = "gloo-net06", not(feature = "wasm")))]
pub use gloo_net_client::HttpClient;

#[cfg(feature = "surf2")]
mod surf_client;
#[cfg(feature = "surf2")]
pub use surf_client::HttpClient;

// Provide a stub when no features are enabled
#[cfg(not(any(feature = "reqwest01", feature = "gloo-net06", feature = "surf2")))]
#[derive(Debug, Clone)]
pub struct HttpClient;

#[cfg(not(any(feature = "reqwest01", feature = "gloo-net06", feature = "surf2")))]
impl HttpClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_bytes(&self, _url: &str) -> Result<bytes::Bytes, error::FetchError> {
        Err(error::FetchError::FetchFailed {
            reason: "No HTTP client feature enabled".to_string(),
        })
    }
}
