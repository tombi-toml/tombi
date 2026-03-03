use tombi_schema_store::DocumentSchema;
use tombi_uri::SchemaUri;

static COMMENT_DIRECTIVE_SCHEMA_STORE: tokio::sync::OnceCell<tombi_schema_store::SchemaStore> =
    tokio::sync::OnceCell::const_new();

#[inline]
pub async fn schema_store() -> &'static tombi_schema_store::SchemaStore {
    COMMENT_DIRECTIVE_SCHEMA_STORE
        .get_or_init(|| async {
            tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
                strict: Some(false),
                ..Default::default()
            })
        })
        .await
}

pub async fn comment_directive_document_schema(
    store: &tombi_schema_store::SchemaStore,
    schema_uri: SchemaUri,
) -> DocumentSchema {
    let tombi_json::ValueNode::Object(object) = store
        .fetch_schema_value(&schema_uri)
        .await
        // Value Comment Directive Schema is embedded in the crate
        .unwrap()
        .unwrap()
    else {
        panic!(
            "Failed to fetch value comment directive schema from URL '{schema_uri}'. \
             The fetched value was not an object."
        );
    };
    DocumentSchema::new(object, schema_uri)
}
