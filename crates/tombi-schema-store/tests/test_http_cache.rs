#![cfg(not(target_arch = "wasm32"))]

use std::{
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use rstest::rstest;
use tombi_schema_store::{FetchError, HttpClient, HttpFuture, SchemaStore, SchemaUri};
use tombi_test_lib::TestCacheHome;

const SCHEMA: &[u8] = br#"{"type":"object"}"#;

#[derive(Debug)]
struct ControlledHttpClient {
    response: Result<Bytes, FetchError>,
    calls: AtomicUsize,
}

impl ControlledHttpClient {
    fn new(response: Result<Bytes, FetchError>) -> Self {
        Self {
            response,
            calls: AtomicUsize::new(0),
        }
    }
}

impl HttpClient for ControlledHttpClient {
    fn get_bytes<'a>(&'a self, _url: &'a str) -> HttpFuture<'a, Result<Bytes, FetchError>> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.response.clone()
        })
    }
}

fn store(client: Arc<ControlledHttpClient>, ttl: Duration) -> SchemaStore {
    SchemaStore::new_with_options_and_http_client(
        tombi_schema_store::Options {
            cache: Some(tombi_cache::Options {
                no_cache: Some(false),
                cache_ttl: Some(ttl),
            }),
            ..Default::default()
        },
        client,
    )
}

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn authentication_failure_does_not_use_stale_cache() {
    let _cache_home = TestCacheHome::new();
    let uri = SchemaUri::from_str("https://example.com/cache/schema.json").unwrap();
    let cache_path = tombi_cache::get_cache_file_path(&uri).await.unwrap();
    tombi_cache::save_to_cache(Some(&cache_path), SCHEMA)
        .await
        .unwrap();
    let client = Arc::new(ControlledHttpClient::new(Err(
        FetchError::AuthenticationFailed {
            reason: "denied".into(),
        },
    )));
    let result = store(client.clone(), Duration::ZERO)
        .fetch_schema_document(&uri)
        .await;

    assert!(matches!(
        result,
        Err(tombi_schema_store::Error::SchemaFetchFailed { .. })
    ));
    assert_eq!(client.calls.load(Ordering::Relaxed), 1);
}

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn policy_denial_does_not_use_cached_schema() {
    let _cache_home = TestCacheHome::new();
    let uri = SchemaUri::from_str("http://127.0.0.1/cache/blocked.json").unwrap();
    let cache_path = tombi_cache::get_cache_file_path(&uri).await.unwrap();
    tombi_cache::save_to_cache(Some(&cache_path), SCHEMA)
        .await
        .unwrap();
    let client = Arc::new(ControlledHttpClient::new(Ok(Bytes::from_static(SCHEMA))));
    let result = store(client.clone(), Duration::ZERO)
        .fetch_schema_document(&uri)
        .await;

    assert!(matches!(
        result,
        Err(tombi_schema_store::Error::SchemaHostNotTrusted { schema_uri }) if schema_uri == uri
    ));
    assert_eq!(client.calls.load(Ordering::Relaxed), 0);
}

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn invalid_json_is_not_persisted() {
    let content = b"<html>Sign in</html>";
    let _cache_home = TestCacheHome::new();
    let uri = SchemaUri::from_str("https://example.com/cache/invalid.json").unwrap();
    let client = Arc::new(ControlledHttpClient::new(Ok(Bytes::from_static(content))));
    let result = store(client, Duration::from_secs(3600))
        .fetch_schema_document(&uri)
        .await;

    assert!(matches!(
        result,
        Err(tombi_schema_store::Error::SchemaFileParseFailed { .. })
    ));
    let cache_path = tombi_cache::get_cache_file_path(&uri).await.unwrap();
    assert!(!cache_path.exists());
}

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn successful_fetch_reuses_fresh_disk_cache() {
    let _cache_home = TestCacheHome::new();
    let uri = SchemaUri::from_str("https://example.com/cache/fresh.json").unwrap();
    let client = Arc::new(ControlledHttpClient::new(Ok(Bytes::from_static(SCHEMA))));
    let schema_store = store(client.clone(), Duration::from_secs(3600));

    assert!(
        schema_store
            .fetch_schema_document(&uri)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        schema_store
            .fetch_schema_document(&uri)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(client.calls.load(Ordering::Relaxed), 1);
    let cache_path = tombi_cache::get_cache_file_path(&uri).await.unwrap();
    assert_eq!(std::fs::read(&cache_path).unwrap(), SCHEMA);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(cache_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
