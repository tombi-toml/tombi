use std::env;
use std::fs;
use std::process;
use std::str::FromStr;

use tombi_json::parse;
use tombi_schema_store::DocumentSchema;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path-or-url>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  # From local file");
        eprintln!("  cargo run --example parse_schema ./tombi.json");
        eprintln!();
        eprintln!("  # From URL");
        eprintln!("  cargo run --example parse_schema https://json.schemastore.org/tombi.json");
        process::exit(1);
    }

    let input = &args[1];

    // Read the JSON content from file or URL
    let (content, schema_uri) = match read_content(input) {
        Ok((content, schema_uri)) => (content, schema_uri),
        Err(err) => {
            eprintln!("Error reading from '{}': {}", input, err);
            process::exit(1);
        }
    };

    // Parse the JSON content
    match parse(&content) {
        Ok(value_node) => {
            eprintln!("✅ Parse successful!");
            let object_node = match value_node {
                tombi_json::ValueNode::Object(object_node) => object_node,
                _ => {
                    eprintln!("❌ Parse error: expected object node");
                    process::exit(1);
                }
            };

            println!("{:#?}", DocumentSchema::new(object_node, schema_uri));
        }
        Err(err) => {
            eprintln!("❌ Parse error: {}", err);
            process::exit(1);
        }
    }
}

/// Read content from a file path or URL
fn read_content(input: &str) -> Result<(String, tombi_uri::SchemaUri), Box<dyn std::error::Error>> {
    if input.starts_with("http://") || input.starts_with("https://") {
        // Read from URL
        eprintln!("📡 Fetching from URL...");
        let response = reqwest::blocking::get(input)?;
        let content = response.text()?;
        Ok((content, tombi_uri::SchemaUri::from_str(input)?))
    } else {
        // Read from file
        eprintln!("📂 Reading from file...");
        let content = fs::read_to_string(input)?;
        Ok((
            content,
            tombi_uri::SchemaUri::from_file_path(std::path::Path::new(input)).map_err(|_| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid file path",
                ))
            })?,
        ))
    }
}
