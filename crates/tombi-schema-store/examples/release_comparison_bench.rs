use std::{env, hint::black_box, str::FromStr, time::Instant};

use tombi_schema_store::{DocumentSchema, SchemaStore, SchemaUri};

fn schema_for(shape: &str) -> String {
    match shape {
        "simple" => r#"{"type":"object","properties":{"value":{"type":"integer"}}}"#.into(),
        "flat" => {
            let properties = (0..50)
                .map(|i| format!(r#""field{i}":{{"type":"integer"}}"#))
                .collect::<Vec<_>>()
                .join(",");
            format!(r#"{{"type":"object","properties":{{{properties}}}}}"#)
        }
        "nested-defs" | "nested-ids" | "nested-anchor" => {
            let mut schema = if shape == "nested-anchor" {
                r##"{"$anchor":"leaf","type":"object"}"##.to_string()
            } else {
                r#"{"type":"object"}"#.to_string()
            };
            for index in (0..50).rev() {
                let id = if shape == "nested-ids" {
                    format!(r#""$id":"r{index}/","#)
                } else {
                    String::new()
                };
                schema = format!(r#"{{{id}"$defs":{{"child":{schema}}}}}"#);
            }
            schema
        }
        _ => panic!("unknown shape"),
    }
}

fn main() {
    let shape = env::args().nth(1).expect("shape argument");
    let schema = schema_for(&shape);
    let uri = SchemaUri::from_str("https://example.com/schema.json").unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut elapsed = std::time::Duration::ZERO;
    for iteration in 0..1020 {
        let schema_document = tombi_json::Document::from_str(&schema).unwrap();
        let store = SchemaStore::new();
        let start = Instant::now();
        let document = runtime.block_on(DocumentSchema::new(
            schema_document,
            uri.clone(),
            None,
            &store,
        ));
        if iteration >= 20 {
            elapsed += start.elapsed();
        }
        black_box(document.expect("schema construction failed"));
    }
    println!(
        "{shape}: {:.3} ms / 1000 loads",
        elapsed.as_secs_f64() * 1000.0
    );
}
