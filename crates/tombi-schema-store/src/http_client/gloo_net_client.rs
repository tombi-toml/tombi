use crate::http_client::{FetchError, HttpClient, HttpFuture};
use bytes::Bytes;
use tombi_future::Boxable;

#[derive(Debug, Clone, Default)]
pub struct GlooNetHttpClient;

impl GlooNetHttpClient {
    pub fn new() -> Self {
        Self
    }
}

impl HttpClient for GlooNetHttpClient {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        async move {
            let response = gloo_net::http::Request::get(url)
                .send()
                .await
                .map_err(|err| FetchError::FetchFailed {
                    reason: err.to_string(),
                })?;

            let is_success = 200 <= response.status() && response.status() < 300;
            if !is_success {
                return Err(FetchError::StatusNotOk {
                    status: response.status(),
                });
            }

            let binary = response
                .binary()
                .await
                .map_err(|e| FetchError::BodyReadFailed {
                    reason: e.to_string(),
                })?;

            Ok(Bytes::from(binary))
        }
        .boxed()
    }
}
