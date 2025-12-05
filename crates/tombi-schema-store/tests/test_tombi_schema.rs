use std::str::FromStr;

use tombi_schema_store::SchemaUri;
use tombi_test_lib::tombi_schema_path;

#[test]
fn tombi_schema() -> Result<(), Box<dyn std::error::Error>> {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    let document_path = tombi_schema_path();
    let file = File::open(&document_path)?;
    let mut reader = BufReader::new(file);

    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let value_node = tombi_json::ValueNode::from_str(&contents)?;
    match value_node {
        tombi_json::ValueNode::Object(_) => Ok(()),
        _ => Err(Box::new(tombi_schema_store::Error::SchemaMustBeObject {
            schema_uri: SchemaUri::from_file_path(&document_path).unwrap(),
        })),
    }
}
