use tombi_schema_store::{SchemaStore, SchemaUri, ValueSchema};

#[tokio::test]
async fn root_ref_resolves_ref_definition_and_keeps_sibling_description() {
    let schema_uri =
        SchemaUri::from_file_path(&tombi_test_lib::root_ref_test_schema_path()).unwrap();
    let schema_store = SchemaStore::new();

    let document_schema = schema_store
        .try_get_document_schema(&schema_uri)
        .await
        .unwrap()
        .unwrap();

    match document_schema.value_schema.as_deref() {
        Some(ValueSchema::Table(_)) => {}
        other => panic!("expected table schema, got {other:?}"),
    }
    assert_eq!(
        document_schema
            .value_schema
            .as_deref()
            .and_then(ValueSchema::description),
        Some("Root override description")
    );
}
