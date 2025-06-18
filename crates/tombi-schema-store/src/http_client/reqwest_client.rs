use crate::http_client::error::FetchError;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct HttpClient(reqwest::Client);

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self(
            reqwest::Client::builder()
                .user_agent("tombi-language-server")
                .build()
                .expect("Failed to create reqwest client"),
        )
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Bytes, FetchError> {
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
}
