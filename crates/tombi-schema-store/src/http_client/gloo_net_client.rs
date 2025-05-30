use crate::http_client::error::FetchError;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Bytes, FetchError> {
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
}
