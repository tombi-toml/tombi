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
