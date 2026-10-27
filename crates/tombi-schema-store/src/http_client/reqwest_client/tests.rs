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
async fn redirect_to_loopback_is_allowed() {
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

    let client = super::ReqwestHttpClient::with_policy(Arc::new(
        crate::schema_fetch_policy::SchemaFetchPolicy::default(),
    ));
    let body = client
        .get_bytes(&format!("http://{address}/first"))
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(body.as_ref(), br#"{"type":"object"}"#);
    assert_eq!(request_count.load(Ordering::Relaxed), 2);
}
