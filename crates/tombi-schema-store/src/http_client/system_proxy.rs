use std::{
    collections::HashSet,
    sync::{Arc, OnceLock},
};

use hyper_util::client::proxy::matcher::Matcher;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

/// Host that `NO_PROXY` is never applied to; used to read back which proxy is
/// configured for each scheme (the exclusion list is kept out of that matcher).
const PROBE_HOST: &str = "tombi-proxy-probe.invalid";

/// Raw proxy settings from one source, in the format of the environment
/// variables (`host:port`, `http://host:port`, `socks5h://host:port`, ...).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ProxySettings {
    all: String,
    http: String,
    https: String,
    no: String,
}

impl ProxySettings {
    /// Same variables and CGI rule as `curl` and reqwest.
    fn from_env() -> Self {
        if std::env::var_os("REQUEST_METHOD").is_some() {
            return Self::default();
        }
        Self {
            all: first_env(&["ALL_PROXY", "all_proxy"]),
            http: first_env(&["HTTP_PROXY", "http_proxy"]),
            https: first_env(&["HTTPS_PROXY", "https_proxy"]),
            no: first_env(&["NO_PROXY", "no_proxy"]),
        }
    }

    fn from_os() -> Self {
        #[cfg(target_os = "macos")]
        return macos::settings();
        #[cfg(windows)]
        return windows::settings();
        #[cfg(not(any(target_os = "macos", windows)))]
        Self::default()
    }

    /// Precedence per scheme: environment scheme-specific, environment `ALL_PROXY`,
    /// then the operating-system setting. `NO_PROXY` from the environment wins
    /// over the operating-system exceptions.
    fn merge(env: Self, os: Self) -> Resolved {
        let pick = |scheme: String, os: String| {
            [scheme, env.all.clone(), os]
                .into_iter()
                .find(|value| !value.trim().is_empty())
                .unwrap_or_default()
        };
        Resolved {
            http: pick(env.http.clone(), os.http),
            https: pick(env.https.clone(), os.https),
            no: if env.no.trim().is_empty() {
                os.no
            } else {
                env.no.clone()
            },
        }
    }
}

fn first_env(names: &[&str]) -> String {
    names
        .iter()
        .find_map(|name| std::env::var(name).ok())
        .unwrap_or_default()
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Resolved {
    http: String,
    https: String,
    no: String,
}

/// Proxies configured for the current user: environment variables first, then
/// the operating-system settings (macOS System Configuration, Windows registry).
///
/// Like `TOMBI_SCHEMA_TRUSTED_HOSTS`, these belong to the user's machine and
/// never come from project files, so the proxy itself may live on a private
/// network.
#[derive(Clone)]
pub(super) struct SystemProxy {
    /// Decides per destination, honoring the exclusion list.
    matcher: Arc<Matcher>,
    /// Same proxies without any exclusion list, to enumerate them.
    unfiltered: Arc<Matcher>,
    hosts: Arc<HashSet<String>>,
}

impl SystemProxy {
    pub(super) fn current() -> &'static Self {
        static CURRENT: OnceLock<SystemProxy> = OnceLock::new();
        CURRENT.get_or_init(|| {
            Self::new(&ProxySettings::merge(
                ProxySettings::from_env(),
                ProxySettings::from_os(),
            ))
        })
    }

    fn new(settings: &Resolved) -> Self {
        let build = |no: &str| {
            Arc::new(
                Matcher::builder()
                    .http(settings.http.trim())
                    .https(settings.https.trim())
                    .no(no)
                    .build(),
            )
        };
        let matcher = build(&settings.no);
        let unfiltered = build("");
        let hosts = ["http", "https"]
            .into_iter()
            .filter_map(|scheme| intercept_uri(&unfiltered, scheme))
            .filter_map(|uri| uri.host().map(normalize_host))
            .collect();

        Self {
            matcher,
            unfiltered,
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
            .filter(|scheme| intercept_uri(&self.unfiltered, scheme).is_some())
            .map(|scheme| {
                let matcher = self.matcher.clone();
                let basic_auth = self
                    .unfiltered
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

/// Windows `ProxyServer`: either `host:port` for every scheme, or
/// `http=host:port;https=host:port` per scheme. Returns `(http, https)`.
#[cfg(any(windows, test))]
fn parse_windows_proxy_server(value: &str) -> (String, String) {
    let value = value.trim();
    if !value.contains('=') {
        return (value.to_string(), value.to_string());
    }

    let mut http = String::new();
    let mut https = String::new();
    for entry in value.split(';') {
        let Some((scheme, server)) = entry.split_once('=') else {
            continue;
        };
        let server = server.trim().to_string();
        match scheme.trim().to_ascii_lowercase().as_str() {
            "http" => http = server,
            "https" => https = server,
            _ => {}
        }
    }
    (http, https)
}

/// Windows `ProxyOverride` (`a.example;*.b.example;<local>`) as a `NO_PROXY` list.
#[cfg(any(windows, test))]
fn windows_proxy_override_to_no_proxy(value: &str) -> String {
    value
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>()
        .join(",")
        .replace("*.", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn host_set(hosts: &[&str]) -> HashSet<String> {
        hosts.iter().map(|host| host.to_string()).collect()
    }

    fn resolved(http: &str, https: &str, no: &str) -> Resolved {
        Resolved {
            http: http.to_string(),
            https: https.to_string(),
            no: no.to_string(),
        }
    }

    fn settings(all: &str, http: &str, https: &str, no: &str) -> ProxySettings {
        ProxySettings {
            all: all.to_string(),
            http: http.to_string(),
            https: https.to_string(),
            no: no.to_string(),
        }
    }

    #[rstest]
    #[case::with_scheme("http://10.0.0.1:3128", &["10.0.0.1"])]
    #[case::without_scheme("proxy.corp.example:3128", &["proxy.corp.example"])]
    #[case::credentials_and_case("http://user:secret@Proxy.Corp.Example.:8080/", &["proxy.corp.example"])]
    #[case::socks("socks5h://proxy.corp.example:1080", &["proxy.corp.example"])]
    #[case::unsupported_scheme("ftp://proxy.corp.example", &[])]
    #[case::empty("", &[])]
    fn hosts_come_from_the_selected_proxies(#[case] value: &str, #[case] expected: &[&str]) {
        let proxy = SystemProxy::new(&resolved(value, value, ""));

        assert_eq!(**proxy.hosts(), host_set(expected));
    }

    #[test]
    fn http_and_https_proxies_are_both_recorded() {
        let proxy = SystemProxy::new(&resolved("http://a.example:1", "http://b.example:2", ""));

        assert_eq!(**proxy.hosts(), host_set(&["a.example", "b.example"]));
    }

    #[rstest]
    #[case::proxied("https://schema.example/a.json", "", true)]
    #[case::no_proxy_exact("https://schema.example/a.json", "schema.example", false)]
    #[case::no_proxy_suffix("https://schema.example/a.json", ".example", false)]
    #[case::other_host("https://other.test/a.json", "schema.example", true)]
    fn no_proxy_is_honored(#[case] url: &str, #[case] no_proxy: &str, #[case] proxied: bool) {
        let proxy = SystemProxy::new(&resolved(
            "http://proxy.corp.example:3128",
            "http://proxy.corp.example:3128",
            no_proxy,
        ));

        assert_eq!(
            proxy.is_proxied(&reqwest::Url::parse(url).unwrap()),
            proxied
        );
    }

    #[rstest]
    #[case::probe_suffix(".invalid")]
    #[case::probe_exact("tombi-proxy-probe.invalid")]
    #[case::wildcard("*")]
    fn exclusions_never_hide_the_configured_proxy(#[case] no_proxy: &str) {
        let proxy = SystemProxy::new(&resolved(
            "http://proxy.corp.example:3128",
            "http://proxy.corp.example:3128",
            no_proxy,
        ));

        assert_eq!(**proxy.hosts(), host_set(&["proxy.corp.example"]));
        assert_eq!(proxy.reqwest_proxies().len(), 2);
    }

    #[test]
    fn each_configured_scheme_gets_a_reqwest_proxy() {
        let proxy = SystemProxy::new(&resolved("", "http://p.example:1", ""));

        assert_eq!(proxy.reqwest_proxies().len(), 1);
    }

    #[rstest]
    #[case::env_scheme_over_env_all(
        settings("http://all:1", "http://env-http:1", "", ""),
        settings("", "os-http:1", "os-https:1", ""),
        resolved("http://env-http:1", "http://all:1", "")
    )]
    #[case::env_all_over_os(
        settings("http://all:1", "", "", ""),
        settings("", "os-http:1", "os-https:1", ""),
        resolved("http://all:1", "http://all:1", "")
    )]
    #[case::os_when_env_is_empty(
        settings("", "", "", ""),
        settings("", "os-http:1", "os-https:1", "os.example"),
        resolved("os-http:1", "os-https:1", "os.example")
    )]
    #[case::env_no_proxy_over_os(
        settings("", "", "", "env.example"),
        settings("", "os-http:1", "", "os.example"),
        resolved("os-http:1", "", "env.example")
    )]
    fn environment_takes_precedence_over_the_operating_system(
        #[case] env: ProxySettings,
        #[case] os: ProxySettings,
        #[case] expected: Resolved,
    ) {
        assert_eq!(ProxySettings::merge(env, os), expected);
    }

    #[rstest]
    #[case::single(
        "proxy.corp.example:8080",
        "proxy.corp.example:8080",
        "proxy.corp.example:8080"
    )]
    #[case::per_scheme(
        "http=a.example:8080;https=b.example:8080",
        "a.example:8080",
        "b.example:8080"
    )]
    #[case::per_scheme_with_unrelated(
        "ftp=f.example:21;https=b.example:8080",
        "",
        "b.example:8080"
    )]
    fn windows_proxy_server_formats(#[case] value: &str, #[case] http: &str, #[case] https: &str) {
        assert_eq!(
            parse_windows_proxy_server(value),
            (http.to_string(), https.to_string())
        );
    }

    #[test]
    fn windows_proxy_override_becomes_a_no_proxy_list() {
        assert_eq!(
            windows_proxy_override_to_no_proxy("a.example; *.b.example;<local>;"),
            "a.example,b.example,<local>"
        );
    }
}
