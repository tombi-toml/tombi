use crate::http_client::{FetchError, HttpClient, HttpFuture, http_timeout_secs};
use bytes::Bytes;
use tombi_future::Boxable;

#[derive(Debug, Clone)]
pub struct ReqwestHttpClient(reqwest::Client);

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self(
            reqwest::Client::builder()
                .user_agent("tombi-language-server")
                .timeout(std::time::Duration::from_secs(http_timeout_secs()))
                .build()
                .expect("Failed to create reqwest client"),
        )
    }
}

impl HttpClient for ReqwestHttpClient {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        async move {
            let response = self
                .0
                .get(url)
                .send()
                .await
                .map_err(|err| FetchError::FetchFailed {
                    reason: err.to_string(),
                })?;

            if !response.status().is_success() {
                return Err(FetchError::StatusNotOk {
                    status: response.status().as_u16(),
                });
            }

            response
                .bytes()
                .await
                .map_err(|err| FetchError::BodyReadFailed {
                    reason: err.to_string(),
                })
        }
        .boxed()
    }
}
