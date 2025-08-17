mod document_tombi_comment_directive;
mod error;

pub use error::Error;
use tombi_schema_store::{CatalogUrl, SchemaUrl, SourceSchema};
use tombi_toml_version::TomlVersion;
use url::Url;

pub const TOMBI_COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

pub use document_tombi_comment_directive::*;

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
static DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URL: std::sync::OnceLock<SchemaUrl> =
    std::sync::OnceLock::new();
static DOCUMENT_COMMENT_DIRECTIVE_SOURCE_SCHEMA: std::sync::OnceLock<SourceSchema> =
    std::sync::OnceLock::new();

#[inline]
pub async fn schema_store() -> &'static tombi_schema_store::SchemaStore {
    COMMENT_DIRECTIVE_SCHEMA_STORE
        .get_or_init(|| async {
            let schema_store = tombi_schema_store::SchemaStore::new();
            let _ = schema_store
                .load_catalog_from_url(&CatalogUrl::new(
                    Url::parse("tombi://json.tombi.dev/api/json/catalog.json").unwrap(),
                ))
                .await;
            schema_store
        })
        .await
}
