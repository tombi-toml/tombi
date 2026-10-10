use crate::http_client::{FetchError, HttpClient, HttpFuture};
use crate::schema_fetch_policy::SchemaFetchPolicy;
use bytes::Bytes;
use std::sync::Arc;
use tombi_future::Boxable;

#[derive(Debug, Clone)]
pub struct GlooNetHttpClient {
    policy: Arc<SchemaFetchPolicy>,
}

impl Default for GlooNetHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GlooNetHttpClient {
    pub fn new() -> Self {
        Self::with_options(&crate::Options::default())
    }

    /// Create a client whose trust policy is the environment plus `options`.
    pub fn with_options(options: &crate::Options) -> Self {
        Self::with_policy(Arc::new(SchemaFetchPolicy::from_options(options)))
    }

    pub(crate) fn with_policy(policy: Arc<SchemaFetchPolicy>) -> Self {
        Self { policy }
    }
}

impl HttpClient for GlooNetHttpClient {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        async move {
            let parsed_url =
                <tombi_uri::Uri as std::str::FromStr>::from_str(url).map_err(|err| {
                    FetchError::FetchFailed {
                        reason: err.to_string(),
                    }
                })?;
            self.policy
                .check_http_url(&parsed_url)
                .map_err(FetchError::from)?;

            let response = gloo_net::http::Request::get(url)
                .redirect(web_sys::RequestRedirect::Error)
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
