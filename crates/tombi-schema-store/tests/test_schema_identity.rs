use std::{io::Write, str::FromStr};

use tombi_schema_store::{DocumentSchema, SchemaStore, SchemaUri};

#[tokio::test]
async fn distinguishes_schema_document_uri_schema_resource_uri_and_optional_id() {
    let schema_document_uri =
        SchemaUri::from_str("https://example.com/schemas/root.json").expect("valid retrieval URI");
    let schema_store = SchemaStore::new();

    let schema_without_id = DocumentSchema::new(
        tombi_json::ValueNode::from_str(r#"{ "type": "object" }"#).expect("valid schema"),
        schema_document_uri.clone(),
        None,
        &schema_store,
    )
    .await
    .expect("DocumentSchema::new");
    assert_eq!(schema_without_id.id, None);
    assert_eq!(
        schema_without_id.schema_document_uri(),
        &schema_document_uri
    );
    assert_eq!(
        schema_without_id.schema_resource_uri(),
        &schema_document_uri
    );
    assert_eq!(schema_without_id.schema_base_uri(), &schema_document_uri);

    let schema_with_id = DocumentSchema::new(
        tombi_json::ValueNode::from_str(r#"{ "$id": "canonical.json" }"#).expect("valid schema"),
        schema_document_uri.clone(),
        None,
        &schema_store,
    )
    .await
    .expect("DocumentSchema::new");
    let canonical_uri = SchemaUri::from_str("https://example.com/schemas/canonical.json")
        .expect("valid canonical URI");
    assert_eq!(schema_with_id.id.as_ref(), Some(&canonical_uri));
    assert_eq!(schema_with_id.schema_document_uri(), &schema_document_uri);
    assert_eq!(schema_with_id.schema_resource_uri(), &canonical_uri);
    assert_eq!(schema_with_id.schema_base_uri(), &canonical_uri);
}

#[tokio::test]
async fn store_rejects_duplicate_canonical_schema_resource_uri() {
    let mut schema_file =
        tempfile::NamedTempFile::with_suffix(".json").expect("temporary schema file");
    schema_file
        .write_all(
            br#"{
                "$defs": {
                    "first": { "$id": "https://example.com/duplicate" },
                    "second": { "$id": "https://example.com/duplicate" }
                }
            }"#,
        )
        .expect("write schema");
    let schema_uri = SchemaUri::from_file_path(schema_file.path()).expect("valid schema file URI");
    let schema_store = SchemaStore::new();

    let error = schema_store
        .try_get_document_schema(&schema_uri)
        .await
        .expect_err("duplicate canonical resource URI");

    assert!(
        error
            .to_string()
            .contains("duplicate $id `https://example.com/duplicate`"),
        "{error}"
    );
    assert!(error.to_string().contains("#/$defs/first"), "{error}");
    assert!(error.to_string().contains("#/$defs/second"), "{error}");
}

#[tokio::test]
async fn reload_config_drops_embedded_resource_index() {
    let mut schema_file =
        tempfile::NamedTempFile::with_suffix(".json").expect("temporary schema file");
    schema_file
        .write_all(
            br#"{
                "$ref": "shoko://embedded/resource",
                "$defs": {
                    "resource": {
                        "$id": "shoko://embedded/resource",
                        "type": "boolean"
                    }
                }
            }"#,
        )
        .expect("write schema");
    let schema_document_uri =
        SchemaUri::from_file_path(schema_file.path()).expect("valid schema file URI");
    let embedded_uri =
        SchemaUri::from_str("shoko://embedded/resource").expect("valid embedded URI");
    let schema_store = SchemaStore::new();

    assert!(
        schema_store
            .try_get_document_schema(&schema_document_uri)
            .await
            .expect("load compound schema")
            .is_some()
    );
    assert!(
        schema_store
            .try_get_document_schema(&embedded_uri)
            .await
            .expect("lookup embedded resource")
            .is_some()
    );

    schema_store
        .reload_config(&tombi_config::Config::default(), None)
        .await
        .expect("reload");

    let lookup = schema_store.try_get_document_schema(&embedded_uri).await;
    assert!(
        lookup.is_err() || lookup.ok().flatten().is_none(),
        "embedded resource must not survive config reload"
    );
}

#[tokio::test]
async fn document_schema_new_registers_embedded_resources_for_offline_refs() {
    let schema_document_uri =
        SchemaUri::from_str("https://example.com/compound.json").expect("valid document URI");
    let embedded_uri = SchemaUri::from_str("shoko://example/resource").expect("valid embedded URI");
    let schema_store = SchemaStore::new();

    let document_schema = DocumentSchema::new(
        tombi_json::ValueNode::from_str(
            r#"{
                "$ref": "shoko://example/resource",
                "$defs": {
                    "resource": {
                        "$id": "shoko://example/resource",
                        "type": "boolean"
                    }
                }
            }"#,
        )
        .expect("valid schema"),
        schema_document_uri,
        None,
        &schema_store,
    )
    .await
    .expect("DocumentSchema::new registers resources");

    assert!(document_schema.schema_view.is_some());
    assert!(
        schema_store
            .try_get_document_schema(&embedded_uri)
            .await
            .expect("lookup embedded resource")
            .is_some()
    );
}

#[tokio::test]
async fn cyclic_embedded_root_refs_do_not_stack_overflow() {
    let mut schema_file =
        tempfile::NamedTempFile::with_suffix(".json").expect("temporary schema file");
    schema_file
        .write_all(
            br#"{
                "$ref": "urn:cycle:a",
                "$defs": {
                    "a": {
                        "$id": "urn:cycle:a",
                        "$ref": "urn:cycle:b"
                    },
                    "b": {
                        "$id": "urn:cycle:b",
                        "$ref": "urn:cycle:a"
                    }
                }
            }"#,
        )
        .expect("write schema");
    let schema_uri = SchemaUri::from_file_path(schema_file.path()).expect("valid schema file URI");
    let schema_store = SchemaStore::new();

    let result = schema_store.try_get_document_schema(&schema_uri).await;
    let document_schema = result
        .expect("cyclic embedded root $refs must not panic")
        .expect("cyclic embedded root $refs must still yield a root document schema");
    assert!(
        document_schema.schema_view.is_some() || document_schema.semantic_schema.is_some(),
        "root document schema should retain a usable view or semantic schema"
    );
}

#[tokio::test]
async fn property_with_non_fragment_id_uses_embedded_resource_base() {
    use std::borrow::Cow;

    use tombi_schema_store::SchemaView;

    let mut schema_file =
        tempfile::NamedTempFile::with_suffix(".json").expect("temporary schema file");
    schema_file
        .write_all(
            br#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": "https://example.com/root.json",
                "type": "object",
                "properties": {
                    "child": {
                        "$id": "https://example.com/child.json",
                        "type": "string"
                    }
                }
            }"#,
        )
        .expect("write schema");
    let schema_document_uri =
        SchemaUri::from_file_path(schema_file.path()).expect("valid schema file URI");
    let child_uri = SchemaUri::from_str("https://example.com/child.json").expect("valid child URI");
    let schema_store = SchemaStore::new();
    let document_schema = schema_store
        .try_get_document_schema(&schema_document_uri)
        .await
        .expect("load schema")
        .expect("schema present");

    let SchemaView::Table(table) = document_schema
        .schema_view
        .as_deref()
        .expect("root table view")
    else {
        panic!("expected table schema view");
    };
    let mut child = table
        .properties
        .read()
        .await
        .get(&tombi_accessor::SchemaAccessor::Key("child".to_string()))
        .expect("child property")
        .property_schema
        .clone();

    let resolved = child
        .resolve(
            Cow::Borrowed(document_schema.schema_base_uri()),
            Cow::Borrowed(&document_schema.definitions),
            None,
            &schema_store,
        )
        .await
        .expect("resolve child resource")
        .expect("child schema");

    assert_eq!(resolved.schema_base_uri.as_ref(), &child_uri);
    assert!(
        matches!(&*resolved.schema_view, SchemaView::String(_)),
        "child resource should expose its own string schema"
    );
}

#[tokio::test]
async fn json_pointer_into_retrieval_uri_uses_canonical_root_base() {
    use std::borrow::Cow;

    use tombi_schema_store::{Referable, ReferenceKind};

    let mut schema_file =
        tempfile::NamedTempFile::with_suffix(".json").expect("temporary schema file");
    schema_file
        .write_all(
            br#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": "https://example.com/canonical.json",
                "type": "object",
                "properties": {
                    "value": { "type": "boolean" }
                }
            }"#,
        )
        .expect("write schema");
    let schema_document_uri =
        SchemaUri::from_file_path(schema_file.path()).expect("valid schema file URI");
    let canonical_uri =
        SchemaUri::from_str("https://example.com/canonical.json").expect("valid canonical URI");
    let schema_store = SchemaStore::new();
    let document_schema = schema_store
        .try_get_document_schema(&schema_document_uri)
        .await
        .expect("load schema")
        .expect("schema present");
    assert_eq!(document_schema.schema_base_uri(), &canonical_uri);

    // Resolve a pointer against the retrieval URI (not the canonical `$id`) to
    // mimic external/file fragment refs that still land in this document.
    let mut via_pointer = Referable::Ref {
        reference: "#/properties/value".to_string(),
        kind: ReferenceKind::Ref,
        semantic_schema: None,
        title: None,
        description: None,
        default: None,
        examples: None,
        deprecation: None,
    };

    let resolved = via_pointer
        .resolve(
            Cow::Owned(schema_document_uri.clone()),
            Cow::Borrowed(&document_schema.definitions),
            None,
            &schema_store,
        )
        .await
        .expect("resolve pointer")
        .expect("pointer target");

    assert_eq!(
        resolved.schema_base_uri.as_ref(),
        &canonical_uri,
        "pointer targets under a retrieval URI must keep the root resource `$id` as base"
    );
}
