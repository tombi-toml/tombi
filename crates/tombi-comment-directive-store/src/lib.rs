use std::str::FromStr;
use tombi_schema_store::DocumentSchema;
use tombi_uri::SchemaUri;

static DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URI: std::sync::OnceLock<SchemaUri> =
    std::sync::OnceLock::new();

static COMMENT_DIRECTIVE_SCHEMA_STORE: tokio::sync::OnceCell<tombi_schema_store::SchemaStore> =
    tokio::sync::OnceCell::const_new();

#[inline]
pub async fn schema_store() -> &'static tombi_schema_store::SchemaStore {
    COMMENT_DIRECTIVE_SCHEMA_STORE
        .get_or_init(|| async {
            let schema_store =
                tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
                    strict: Some(false),
                    ..Default::default()
                });
            schema_store
        })
        .await
}

#[inline]
pub fn document_comment_directive_schema_uri() -> &'static SchemaUri {
    DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URI.get_or_init(|| {
        SchemaUri::from_str("tombi://json.tombi.dev/document-tombi-directive.json").unwrap()
    })
}

pub async fn document_comment_directive_document_schema() -> DocumentSchema {
    let schema_store = schema_store().await;
    let schema_uri = document_comment_directive_schema_uri();
    let tombi_json::ValueNode::Object(object) = schema_store
        .fetch_schema_value(schema_uri)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!(
            "Failed to fetch document comment directive schema from URL '{schema_uri}'. \
             The fetched value was not an object."
        );
    };
    DocumentSchema::new(object, schema_uri.clone())
}
