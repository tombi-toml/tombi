use crate::{
    http_client::{FetchError, HttpClient, HttpFuture, http_timeout_secs},
    schema_fetch_policy::{PolicyError, SchemaFetchPolicy},
};
use bytes::Bytes;
use std::{
    collections::HashSet,
    net::{IpAddr, SocketAddr},
    sync::{Arc, OnceLock},
};
use tombi_future::Boxable;

#[cfg(not(target_arch = "wasm32"))]
use super::github_credentials::github_authorization;

#[derive(Debug, Clone)]
pub struct ReqwestHttpClient {
    client: Arc<OnceLock<Result<reqwest::Client, String>>>,
    policy: Arc<SchemaFetchPolicy>,
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self::with_options(&crate::Options::default())
    }

    /// Create a client whose trust policy is the environment plus
    /// `options.trusted_hosts`.
    pub fn with_options(options: &crate::Options) -> Self {
        Self::with_policy(Arc::new(SchemaFetchPolicy::from_options(options)))
    }

    pub(crate) fn with_policy(policy: Arc<SchemaFetchPolicy>) -> Self {
        Self {
            client: Arc::new(OnceLock::new()),
            policy,
        }
    }

    fn client(&self) -> Result<&reqwest::Client, FetchError> {
        self.client
            .get_or_init(|| build_client(self.policy.clone()).map_err(|err| err.to_string()))
            .as_ref()
            .map_err(|reason| FetchError::FetchFailed {
                reason: reason.clone(),
            })
    }

    async fn get_response(&self, url: reqwest::Url) -> Result<reqwest::Response, FetchError> {
        self.policy
            .check_http_url(url.as_ref())
            .map_err(FetchError::from)?;

        let client = self.client()?;
        let mut current_url = url;
        let mut visited = HashSet::from([current_url.clone()]);

        // reqwest's automatic redirect path hides the destination from the
        // store's stale-cache handling. Follow GET redirects here so a policy
        // refusal remains a typed, terminal error.
        for _ in 0..10 {
            let response = client
                .get(current_url.clone())
                .send()
                .await
                .map_err(fetch_error_from_reqwest)?;

            if !is_redirect_status(response.status()) {
                return Ok(response);
            }

            let Some(location) = response.headers().get(reqwest::header::LOCATION) else {
                return Ok(response);
            };
            let location = location.to_str().map_err(|err| FetchError::FetchFailed {
                reason: format!("invalid redirect location: {err}"),
            })?;
            let next_url =
                response
                    .url()
                    .join(location)
                    .map_err(|err| FetchError::FetchFailed {
                        reason: format!("invalid redirect URL: {err}"),
                    })?;
            self.policy
                .check_http_url(next_url.as_ref())
                .map_err(FetchError::from)?;

            if !visited.insert(next_url.clone()) {
                return Err(FetchError::FetchFailed {
                    reason: "redirect loop detected".to_string(),
                });
            }
            current_url = next_url;
        }

        Err(FetchError::FetchFailed {
            reason: "too many redirects".to_string(),
        })
    }
}

impl HttpClient for ReqwestHttpClient {
    fn get_bytes<'a>(&'a self, url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        async move {
            let parsed_url = reqwest::Url::parse(url).map_err(|err| FetchError::FetchFailed {
                reason: err.to_string(),
            })?;
            let mut response = self.get_response(parsed_url.clone()).await?;

            #[cfg(not(target_arch = "wasm32"))]
            if should_retry_with_github_auth(&parsed_url, response.url(), response.status())
                && let Some(authorization) = github_authorization().await.map_err(|error| {
                    FetchError::AuthenticationFailed {
                        reason: error.to_string(),
                    }
                })?
            {
                response =
                    authenticated_response(parsed_url, authorization, self.policy.clone()).await?;
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

fn build_client(policy: Arc<SchemaFetchPolicy>) -> Result<reqwest::Client, FetchError> {
    let mut builder = reqwest::Client::builder()
        .user_agent("tombi")
        .timeout(std::time::Duration::from_secs(http_timeout_secs()))
        // An ambient proxy can resolve the destination outside our resolver.
        .no_proxy()
        // Redirects are handled manually so each hop is revalidated and policy
        // denials can never be mistaken for ordinary network failures.
        .redirect(reqwest::redirect::Policy::none());

    #[cfg(not(target_arch = "wasm32"))]
    {
        builder = builder.dns_resolver(Arc::new(FilteredDnsResolver { policy }));
    }

    #[cfg(target_arch = "wasm32")]
    let _ = policy;

    builder.build().map_err(|err| FetchError::FetchFailed {
        reason: err.to_string(),
    })
}

fn is_redirect_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308)
}

fn fetch_error_from_reqwest(error: reqwest::Error) -> FetchError {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut source = std::error::Error::source(&error);
        while let Some(cause) = source {
            if let Some(policy_error) = cause.downcast_ref::<PolicyError>() {
                return FetchError::from(policy_error.clone());
            }
            source = std::error::Error::source(cause);
        }
    }

    FetchError::FetchFailed {
        reason: error.to_string(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct FilteredDnsResolver {
    policy: Arc<SchemaFetchPolicy>,
}

#[cfg(not(target_arch = "wasm32"))]
impl reqwest::dns::Resolve for FilteredDnsResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let policy = self.policy.clone();
        let host = name.as_str().to_string();

        Box::pin(async move {
            let addresses = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send + Sync>)?
                .collect::<Vec<SocketAddr>>();
            let resolved_ips = addresses
                .iter()
                .map(SocketAddr::ip)
                .collect::<Vec<IpAddr>>();
            policy
                .check_resolved_addresses(&host, &resolved_ips)
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send + Sync>)?;

            Ok(Box::new(addresses.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn authenticated_response(
    url: reqwest::Url,
    authorization: reqwest::header::HeaderValue,
    policy: Arc<SchemaFetchPolicy>,
) -> Result<reqwest::Response, FetchError> {
    policy
        .check_http_url(url.as_ref())
        .map_err(FetchError::from)?;
    let client = build_client(policy).map_err(|_| FetchError::AuthenticationFailed {
        reason: "failed to create authenticated GitHub HTTP client".to_string(),
    })?;
    let response = client
        .get(url)
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .map_err(|error| match fetch_error_from_reqwest(error) {
            error if error.is_policy_denied() => error,
            _ => FetchError::AuthenticationFailed {
                reason: "authenticated GitHub schema request failed".to_string(),
            },
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
