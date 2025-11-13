use std::env;
use std::fs;
use std::process;

use tombi_json::parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path-or-url>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  # From local file");
        eprintln!("  cargo run --example parse_schema json.schemastore.org/tombi.json");
        eprintln!();
        eprintln!("  # From URL");
        eprintln!("  cargo run --example parse_schema https://json.schemastore.org/tombi.json");
        process::exit(1);
    }

    let input = &args[1];

    // Read the JSON content from file or URL
    let content = match read_content(input) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading from '{}': {}", input, err);
            process::exit(1);
        }
    };

    println!("Parsing JSON Schema: {}", input);
    println!("File size: {} bytes", content.len());
    println!();

    // Parse the JSON content
    match parse(&content) {
        Ok(value_node) => {
            eprintln!("✅ Parse successful!");
            println!("{:#?}", value_node);
        }
        Err(err) => {
            eprintln!("❌ Parse error: {}", err);
            process::exit(1);
        }
    }
}

/// Read content from a file path or URL
fn read_content(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    if input.starts_with("http://") || input.starts_with("https://") {
        // Read from URL
        println!("📡 Fetching from URL...");
        let response = reqwest::blocking::get(input)?;
        let content = response.text()?;
        Ok(content)
    } else {
        // Read from file
        println!("📂 Reading from file...");
        let content = fs::read_to_string(input)?;
        Ok(content)
    }
}
