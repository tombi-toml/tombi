use crate::HttpClient;
use reqwest::{StatusCode, Url};
use rstest::rstest;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use super::should_retry_with_github_auth;

#[rstest]
#[case::private_raw(
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    404,
    true
)]
#[case::anonymous_rate_limit(
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    403,
    true
)]
#[case::other_host(
    "https://example.com/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    404,
    false
)]
#[case::redirected_elsewhere(
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://example.com/schema.json",
    404,
    false
)]
#[case::insecure_origin(
    "http://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    404,
    false
)]
#[case::server_error(
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    500,
    false
)]
fn only_retry_github_raw_authentication_failures(
    #[case] original: &str,
    #[case] response: &str,
    #[case] status: u16,
    #[case] expected: bool,
) {
    assert_eq!(
        should_retry_with_github_auth(
            &Url::parse(original).unwrap(),
            &Url::parse(response).unwrap(),
            StatusCode::from_u16(status).unwrap(),
        ),
        expected,
    );
}

#[tokio::test(flavor = "current_thread")]
async fn redirect_to_a_private_address_is_denied_before_the_second_request() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let request_count = Arc::new(AtomicUsize::new(0));
    let server_request_count = request_count.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        server_request_count.fetch_add(1, Ordering::Relaxed);
        let mut request = [0; 1024];
        let _ = stream.read(&mut request);
        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nLocation: http://169.254.169.254/latest/meta-data/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
    });

    let policy = Arc::new(crate::schema_fetch_policy::SchemaFetchPolicy::from_options(
        &crate::Options {
            trusted_hosts: Some(vec!["127.0.0.1".to_string()]),
            ..Default::default()
        },
    ));
    let client = super::ReqwestHttpClient::with_policy(policy);
    let result = client
        .get_bytes(&format!("http://{address}/redirect"))
        .await;
    server.join().unwrap();

    assert!(matches!(result, Err(crate::FetchError::HostNotTrusted)));
    assert_eq!(request_count.load(Ordering::Relaxed), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn redirects_within_an_explicitly_trusted_host_remain_supported() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let request_count = Arc::new(AtomicUsize::new(0));
    let server_request_count = request_count.clone();
    let server = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            server_request_count.fetch_add(1, Ordering::Relaxed);
            let mut request = [0; 1024];
            let length = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..length]);
            if request.starts_with("GET /first ") {
                stream
                    .write_all(
                        b"HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .unwrap();
            } else {
                let body = br#"{"type":"object"}"#;
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
                .unwrap();
                stream.write_all(body).unwrap();
            }
        }
    });

    let policy = Arc::new(crate::schema_fetch_policy::SchemaFetchPolicy::from_options(
        &crate::Options {
            trusted_hosts: Some(vec!["127.0.0.1".to_string()]),
            ..Default::default()
        },
    ));
    let client = super::ReqwestHttpClient::with_policy(policy);
    let body = client
        .get_bytes(&format!("http://{address}/first"))
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(body.as_ref(), br#"{"type":"object"}"#);
    assert_eq!(request_count.load(Ordering::Relaxed), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn private_dns_answers_are_reported_as_policy_denials() {
    let client = super::ReqwestHttpClient::with_policy(Arc::new(
        crate::schema_fetch_policy::SchemaFetchPolicy::default(),
    ));
    let result = client.get_bytes("http://localhost/schema.json").await;

    assert!(matches!(result, Err(crate::FetchError::HostNotTrusted)));
}

#[rstest]
#[case::configured_proxy(&["localhost"], true)]
#[case::any_other_host(&["proxy.corp.example"], false)]
#[tokio::test(flavor = "current_thread")]
async fn only_configured_proxy_hosts_may_resolve_to_private_addresses(
    #[case] proxy_hosts: &[&str],
    #[case] allowed: bool,
) {
    use reqwest::dns::Resolve;
    use std::str::FromStr;

    let resolver = super::FilteredDnsResolver {
        policy: Arc::new(crate::schema_fetch_policy::SchemaFetchPolicy::default()),
        proxy_hosts: Arc::new(proxy_hosts.iter().map(|host| host.to_string()).collect()),
    };
    let result = resolver
        .resolve(reqwest::dns::Name::from_str("localhost").unwrap())
        .await;

    assert_eq!(result.is_ok(), allowed);
}

#[rstest]
#[case::same_name("http://proxy.corp.example/schema.json", false)]
#[case::case_and_trailing_dot("http://Proxy.Corp.Example./schema.json", false)]
#[case::other_host("http://schema.example/schema.json", true)]
fn schema_urls_must_not_target_the_proxy_host(#[case] url: &str, #[case] allowed: bool) {
    let proxy_hosts = std::collections::HashSet::from(["proxy.corp.example".to_string()]);
    let result = super::check_not_proxy_host(&proxy_hosts, &reqwest::Url::parse(url).unwrap());

    assert_eq!(result.is_ok(), allowed);
}

#[rstest]
#[case::resolves_to_loopback("http://localhost/schema.json", false)]
#[case::ip_literal_is_left_to_url_check("http://127.0.0.1/schema.json", true)]
#[case::unresolvable_is_left_to_the_proxy("http://tombi-unresolvable.invalid/schema.json", true)]
#[tokio::test(flavor = "current_thread")]
async fn proxied_destinations_are_checked_locally(#[case] url: &str, #[case] allowed: bool) {
    let policy = crate::schema_fetch_policy::SchemaFetchPolicy::default();
    let result = super::check_local_resolution(&policy, &reqwest::Url::parse(url).unwrap()).await;

    assert_eq!(result.is_ok(), allowed);
}
