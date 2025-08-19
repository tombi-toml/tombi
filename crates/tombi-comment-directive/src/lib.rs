mod document;
mod error;
mod value;

use std::str::FromStr;

pub use error::Error;
use tombi_schema_store::{CatalogUri, SchemaUri, SourceSchema};
use tombi_toml_version::TomlVersion;

pub const TOMBI_COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

pub use document::*;

fn into_directive_diagnostic(
    diagnostic: &tombi_diagnostic::Diagnostic,
    content_range: tombi_text::Range,
) -> tombi_diagnostic::Diagnostic {
    tombi_diagnostic::Diagnostic::new_warning(
        diagnostic.message(),
        diagnostic.code(),
        tombi_text::Range::new(
            content_range.start + tombi_text::RelativePosition::from(diagnostic.range().start),
            content_range.start + tombi_text::RelativePosition::from(diagnostic.range().end),
        ),
    )
}

static COMMENT_DIRECTIVE_SCHEMA_STORE: tokio::sync::OnceCell<tombi_schema_store::SchemaStore> =
    tokio::sync::OnceCell::const_new();
static DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URI: std::sync::OnceLock<SchemaUri> =
    std::sync::OnceLock::new();
static DOCUMENT_COMMENT_DIRECTIVE_SOURCE_SCHEMA: std::sync::OnceLock<SourceSchema> =
    std::sync::OnceLock::new();

#[inline]
pub async fn schema_store() -> &'static tombi_schema_store::SchemaStore {
    COMMENT_DIRECTIVE_SCHEMA_STORE
        .get_or_init(|| async {
            let schema_store = tombi_schema_store::SchemaStore::new();
            let _ = schema_store
                .load_catalog_from_uri(&CatalogUri::new(
                    tombi_uri::Uri::from_str("tombi://json.tombi.dev/api/json/catalog.json")
                        .unwrap(),
                ))
                .await;
            schema_store
        })
        .await
}
