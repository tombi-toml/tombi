#![cfg(not(target_arch = "wasm32"))]

use std::{
    io::{Read, Write},
    net::TcpListener,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use rstest::rstest;
use tombi_schema_store::{Error, Options, SchemaStore, SchemaUri};
use tombi_test_lib::TestCacheHome;

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn schema_fetch_does_not_connect_to_loopback_addresses() {
    let _cache_home = TestCacheHome::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let stop_server = Arc::new(AtomicBool::new(false));
    let request_received = Arc::new(AtomicBool::new(false));
    let server_stop = stop_server.clone();
    let server_request_received = request_received.clone();
    let server = thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    server_request_received.store(true, Ordering::Relaxed);
                    let mut request = [0; 1024];
                    let _ = stream.read(&mut request);
                    let body = br#"{"type":"object"}"#;
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .unwrap();
                    stream.write_all(body).unwrap();
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if server_stop.load(Ordering::Relaxed) {
                        return;
                    }
                    thread::sleep(Duration::from_millis(2));
                }
                Err(_) => return,
            }
        }
    });

    let store = SchemaStore::new_with_options(Options {
        cache: Some(tombi_cache::Options {
            no_cache: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });
    let mut results = Vec::new();
    for host in ["127.0.0.1", "localhost"] {
        let uri =
            SchemaUri::from_str(&format!("http://{host}:{}/schema.json", address.port())).unwrap();
        results.push(store.fetch_schema_document(&uri).await);
    }
    stop_server.store(true, Ordering::Relaxed);
    server.join().unwrap();

    assert!(
        results
            .iter()
            .all(|result| matches!(result, Err(Error::SchemaHostNotTrusted { .. })))
    );
    assert!(!request_received.load(Ordering::Relaxed));
}
