use std::path::PathBuf;

use tombi_schema_store::{DocumentSchema, SchemaUrl};

fn project_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cargo_manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

    if cargo_manifest_dir.ends_with("tombi-schema-store") {
        Ok(cargo_manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf())
    } else {
        Ok(cargo_manifest_dir)
    }
}

#[test]
fn tombi_schema() -> Result<(), Box<dyn std::error::Error>> {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    let document_path = project_root()?.join("tombi.schema.json");
    let file = File::open(&document_path)?;
    let mut reader = BufReader::new(file);

    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let document_schema = DocumentSchema::new(
        tombi_json_value::from_str(&contents)?,
        SchemaUrl::from_file_path(&document_path).unwrap(),
    );

    dbg!(document_schema);
    Ok(())
}
