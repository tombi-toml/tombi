use std::{hint::black_box, str::FromStr, time::Instant};

use tombi_schema_store::{DocumentSchema, SchemaStore, SchemaUri};

fn main() {
    let mut schema = r#"{"type":"object"}"#.to_string();
    for index in (0..50).rev() {
        schema = format!(r#"{{"$id":"r{index}/","$defs":{{"child":{schema}}}}}"#);
    }
    let uri = SchemaUri::from_str("https://example.com/schema.json").unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut elapsed = std::time::Duration::ZERO;
    for iteration in 0..105 {
        let schema_document = tombi_json::Document::from_str(&schema).unwrap();
        let store = SchemaStore::new();
        let start = Instant::now();
        let document = runtime
            .block_on(DocumentSchema::new(
                schema_document,
                uri.clone(),
                None,
                &store,
            ))
            .unwrap();
        if iteration >= 5 {
            elapsed += start.elapsed();
        }
        black_box(document);
    }

    println!(
        "100 loads of 50 nested resources: {:.3} ms",
        elapsed.as_secs_f64() * 1000.0
    );
}
