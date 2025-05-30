use crate::http_client::error::FetchError;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct HttpClient(surf::Client);

impl HttpClient {
    pub fn new() -> Self {
        Self(surf::Client::new())
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Bytes, FetchError> {
        let mut response = self
            .0
            .get(url)
            .send()
            .await
            .map_err(|err| FetchError::FetchFailed {
                reason: err.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(FetchError::StatusNotOk {
                status: response.status().into(),
            });
        }

        let binary = response
            .body_bytes()
            .await
            .map_err(|e| FetchError::BodyReadFailed {
                reason: e.to_string(),
            })?;

        Ok(Bytes::from(binary))
    }
}
