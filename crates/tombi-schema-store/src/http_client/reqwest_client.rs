use crate::http_client::{FetchError, HttpClient, HttpFuture, http_timeout_secs};
use bytes::Bytes;
use std::sync::{Arc, OnceLock};
use tombi_future::Boxable;

#[derive(Debug, Clone)]
pub struct ReqwestHttpClient(Arc<OnceLock<Result<reqwest::Client, String>>>);

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self(Arc::new(OnceLock::new()))
    }

    fn client(&self) -> Result<&reqwest::Client, FetchError> {
        self.0
            .get_or_init(|| {
                reqwest::Client::builder()
                    .user_agent("tombi-language-server")
                    .timeout(std::time::Duration::from_secs(http_timeout_secs()))
                    .build()
                    .map_err(|err| err.to_string())
            })
            .as_ref()
            .map_err(|reason| FetchError::FetchFailed {
                reason: reason.clone(),
            })
    }
}

impl HttpClient for ReqwestHttpClient {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        async move {
            let response =
                self.client()?
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
