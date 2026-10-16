use std::{
    collections::HashSet,
    sync::{Arc, OnceLock},
};

use hyper_util::client::proxy::matcher::Matcher;

/// Host that no `NO_PROXY` rule is expected to name; used to read back which
/// proxy a matcher selects for each scheme.
const PROBE_HOST: &str = "tombi-proxy-probe.invalid";

/// Proxies configured for the current user: environment variables first, then
/// the operating-system settings (macOS System Configuration, Windows registry).
///
/// Like `TOMBI_SCHEMA_TRUSTED_HOSTS`, these belong to the user's machine and
/// never come from project files, so the proxy itself may live on a private
/// network.
#[derive(Clone)]
pub(super) struct SystemProxy {
    matcher: Arc<Matcher>,
    hosts: Arc<HashSet<String>>,
}

impl SystemProxy {
    pub(super) fn current() -> &'static Self {
        static CURRENT: OnceLock<SystemProxy> = OnceLock::new();
        CURRENT.get_or_init(|| Self::new(Matcher::from_system()))
    }

    fn new(matcher: Matcher) -> Self {
        let hosts = ["http", "https"]
            .into_iter()
            .filter_map(|scheme| intercept_uri(&matcher, scheme))
            .filter_map(|uri| uri.host().map(normalize_host))
            .collect();

        Self {
            matcher: Arc::new(matcher),
            hosts: Arc::new(hosts),
        }
    }

    /// Hosts of the configured proxies, lowercased and without a trailing dot.
    pub(super) fn hosts(&self) -> &Arc<HashSet<String>> {
        &self.hosts
    }

    /// Whether a request to `url` is sent through a proxy (honoring `NO_PROXY`).
    pub(super) fn is_proxied(&self, url: &reqwest::Url) -> bool {
        url.as_str()
            .parse::<http::Uri>()
            .ok()
            .is_some_and(|uri| self.matcher.intercept(&uri).is_some())
    }

    /// reqwest proxies equivalent to this configuration. The client must be
    /// built with `no_proxy()` so reqwest does not read its own copy.
    pub(super) fn reqwest_proxies(&self) -> Vec<reqwest::Proxy> {
        ["http", "https"]
            .into_iter()
            .filter(|scheme| intercept_uri(&self.matcher, scheme).is_some())
            .map(|scheme| {
                let matcher = self.matcher.clone();
                let basic_auth = matcher
                    .intercept(&probe_uri(scheme))
                    .and_then(|intercept| intercept.basic_auth().cloned());
                let proxy = reqwest::Proxy::custom(move |url| {
                    if url.scheme() != scheme {
                        return None;
                    }
                    let uri = url.as_str().parse::<http::Uri>().ok()?;
                    let intercept = matcher.intercept(&uri)?;
                    let mut proxy_url = reqwest::Url::parse(&intercept.uri().to_string()).ok()?;
                    if let Some((user, password)) = intercept.raw_auth() {
                        proxy_url.set_username(user).ok()?;
                        proxy_url.set_password(Some(password)).ok()?;
                    }
                    Some(proxy_url)
                });
                match basic_auth {
                    Some(value) => proxy.custom_http_auth(value),
                    None => proxy,
                }
            })
            .collect()
    }
}

fn probe_uri(scheme: &str) -> http::Uri {
    format!("{scheme}://{PROBE_HOST}/")
        .parse()
        .expect("probe URI is valid")
}

fn intercept_uri(matcher: &Matcher, scheme: &str) -> Option<http::Uri> {
    matcher
        .intercept(&probe_uri(scheme))
        .map(|intercept| intercept.uri().clone())
}

pub(super) fn normalize_host(host: &str) -> String {
    host.trim_end_matches('.').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn host_set(hosts: &[&str]) -> HashSet<String> {
        hosts.iter().map(|host| host.to_string()).collect()
    }

    #[rstest]
    #[case::with_scheme("http://10.0.0.1:3128", &["10.0.0.1"])]
    #[case::without_scheme("proxy.corp.example:3128", &["proxy.corp.example"])]
    #[case::credentials_and_case("http://user:secret@Proxy.Corp.Example.:8080/", &["proxy.corp.example"])]
    #[case::socks("socks5h://proxy.corp.example:1080", &["proxy.corp.example"])]
    #[case::unsupported_scheme("ftp://proxy.corp.example", &[])]
    #[case::empty("", &[])]
    fn hosts_come_from_the_selected_proxies(#[case] value: &str, #[case] expected: &[&str]) {
        let proxy = SystemProxy::new(Matcher::builder().all(value).build());

        assert_eq!(**proxy.hosts(), host_set(expected));
    }

    #[test]
    fn http_and_https_proxies_are_both_recorded() {
        let proxy = SystemProxy::new(
            Matcher::builder()
                .http("http://a.example:1")
                .https("http://b.example:2")
                .build(),
        );

        assert_eq!(**proxy.hosts(), host_set(&["a.example", "b.example"]));
    }

    #[rstest]
    #[case::proxied("https://schema.example/a.json", "", true)]
    #[case::no_proxy_exact("https://schema.example/a.json", "schema.example", false)]
    #[case::no_proxy_suffix("https://schema.example/a.json", ".example", false)]
    #[case::other_host("https://other.test/a.json", "schema.example", true)]
    fn no_proxy_is_honored(#[case] url: &str, #[case] no_proxy: &str, #[case] proxied: bool) {
        let proxy = SystemProxy::new(
            Matcher::builder()
                .all("http://proxy.corp.example:3128")
                .no(no_proxy)
                .build(),
        );

        assert_eq!(
            proxy.is_proxied(&reqwest::Url::parse(url).unwrap()),
            proxied
        );
    }

    #[test]
    fn each_configured_scheme_gets_a_reqwest_proxy() {
        let proxy = SystemProxy::new(Matcher::builder().https("http://p.example:1").build());

        assert_eq!(proxy.reqwest_proxies().len(), 1);
    }
}
