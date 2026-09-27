use std::sync::Arc;

use tombi_schema_store::DocumentSchema;
use tombi_uri::SchemaUri;

static COMMENT_DIRECTIVE_SCHEMA_STORE: tokio::sync::OnceCell<tombi_schema_store::SchemaStore> =
    tokio::sync::OnceCell::const_new();

#[inline]
pub async fn schema_store() -> &'static tombi_schema_store::SchemaStore {
    COMMENT_DIRECTIVE_SCHEMA_STORE
        .get_or_init(|| async {
            tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
                strict: Some(false.into()),
                ..Default::default()
            })
        })
        .await
}

pub async fn comment_directive_document_schema(
    store: &tombi_schema_store::SchemaStore,
    schema_uri: SchemaUri,
) -> Arc<DocumentSchema> {
    store
        .try_get_document_schema(&schema_uri)
        .await
        .expect("failed to load embedded comment directive schema")
        .expect("embedded comment directive schema is missing")
}
