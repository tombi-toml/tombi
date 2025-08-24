mod document;
mod error;
mod value;

use std::str::FromStr;

pub use error::Error;
use tombi_schema_store::{CatalogUri, SchemaUri, SourceSchema};
use tombi_toml_version::TomlVersion;

pub const TOMBI_COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

pub use document::*;
pub use value::*;

static COMMENT_DIRECTIVE_SCHEMA_STORE: tokio::sync::OnceCell<tombi_schema_store::SchemaStore> =
    tokio::sync::OnceCell::const_new();
static COMMENT_DIRECTIVE_SOURCE_SCHEMA: std::sync::OnceLock<SourceSchema> =
    std::sync::OnceLock::new();
static DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URI: std::sync::OnceLock<SchemaUri> =
    std::sync::OnceLock::new();

#[inline]
pub async fn schema_store() -> &'static tombi_schema_store::SchemaStore {
    COMMENT_DIRECTIVE_SCHEMA_STORE
        .get_or_init(|| async {
            let schema_store = tombi_schema_store::SchemaStore::new();
            schema_store
        })
        .await
}

#[inline]
pub async fn source_schema(
    document_schema: tombi_schema_store::DocumentSchema,
) -> &'static SourceSchema {
    COMMENT_DIRECTIVE_SOURCE_SCHEMA.get_or_init(|| tombi_schema_store::SourceSchema {
        root_schema: Some(document_schema),
        sub_schema_uri_map: ahash::AHashMap::with_capacity(0),
    })
}
