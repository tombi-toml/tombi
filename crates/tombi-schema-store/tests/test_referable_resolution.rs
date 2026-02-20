use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tombi_schema_store::{SchemaStore, SchemaUri};

fn unique_temp_path(file_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();

    std::env::temp_dir()
        .join(format!("tombi-schema-store-{nanos}-{}", std::process::id()))
        .join(file_name)
}

#[tokio::test]
async fn local_ref_resolution_does_not_reenter_same_schema_fetch()
-> Result<(), Box<dyn std::error::Error>> {
    let schema_path = unique_temp_path("recursive-oneof.json");
    if let Some(parent) = schema_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(
        &schema_path,
        r##"{
  "oneOf": [
    { "$ref": "#/definitions/value" }
  ],
  "definitions": {
    "value": { "type": "string" }
  }
}"##,
    )?;

    let schema_store = SchemaStore::new();
    let schema_uri = SchemaUri::from_file_path(&schema_path)
        .map_err(|_| format!("failed to build schema uri for {}", schema_path.display()))?;

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        schema_store.try_get_document_schema(&schema_uri),
    )
    .await;

    let document_schema = result.map_err(|_| "schema resolution timed out")??;
    assert!(document_schema.is_some());

    if let Some(parent) = schema_path.parent() {
        let _ = fs::remove_dir_all(parent);
    }

    Ok(())
}
