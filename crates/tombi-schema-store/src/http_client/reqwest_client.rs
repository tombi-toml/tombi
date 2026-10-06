use crate::http_client::{FetchError, HttpClient, HttpFuture, http_timeout_secs};
use bytes::Bytes;
use std::sync::{Arc, OnceLock};
use tombi_future::Boxable;

#[cfg(not(target_arch = "wasm32"))]
use super::github_credentials::github_authorization;

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
                    .user_agent("tombi")
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
            let mut response =
                self.client()?
                    .get(url)
                    .send()
                    .await
                    .map_err(|err| FetchError::FetchFailed {
                        reason: err.to_string(),
                    })?;

            #[cfg(not(target_arch = "wasm32"))]
            if let Ok(original_url) = reqwest::Url::parse(url)
                && should_retry_with_github_auth(&original_url, response.url(), response.status())
                && let Some(authorization) = github_authorization().await.map_err(|error| {
                    FetchError::AuthenticationFailed {
                        reason: error.to_string(),
                    }
                })?
            {
                response = authenticated_response(original_url, authorization).await?;
            }

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

#[cfg(not(target_arch = "wasm32"))]
async fn authenticated_response(
    url: reqwest::Url,
    authorization: reqwest::header::HeaderValue,
) -> Result<reqwest::Response, FetchError> {
    let client = reqwest::Client::builder()
        .user_agent("tombi")
        .timeout(std::time::Duration::from_secs(http_timeout_secs()))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| FetchError::AuthenticationFailed {
            reason: "failed to create authenticated GitHub HTTP client".to_string(),
        })?;
    let response = client
        .get(url)
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .map_err(|_| FetchError::AuthenticationFailed {
            reason: "authenticated GitHub schema request failed".to_string(),
        })?;
    if !response.status().is_success() {
        return Err(FetchError::AuthenticationFailed {
            reason: format!("unexpected status: {}", response.status().as_u16()),
        });
    }
    Ok(response)
}

#[cfg(not(target_arch = "wasm32"))]
fn is_github_raw_url(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("raw.githubusercontent.com")
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
}

#[cfg(not(target_arch = "wasm32"))]
fn should_retry_with_github_auth(
    original_url: &reqwest::Url,
    response_url: &reqwest::Url,
    status: reqwest::StatusCode,
) -> bool {
    is_github_raw_url(original_url)
        && is_github_raw_url(response_url)
        && matches!(status.as_u16(), 401 | 403 | 404)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
