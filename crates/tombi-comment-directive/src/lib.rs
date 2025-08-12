mod error;
mod root;

use ahash::AHashMap;
pub use error::Error;
pub use root::RootTombiDirective;
use tombi_ast::TombiCommentDirective;
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::{CatalogUrl, DocumentSchema, SchemaUrl, SourceSchema};
use tombi_toml_version::TomlVersion;
use url::Url;

pub const TOMBI_COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

pub async fn get_root_comment_directive(root: &tombi_ast::Root) -> Option<RootTombiDirective> {
    try_get_root_comment_directive(root).await.ok().flatten()
}

pub async fn try_get_root_comment_directive(
    root: &tombi_ast::Root,
) -> Result<Option<RootTombiDirective>, Vec<tombi_diagnostic::Diagnostic>> {
    use serde::Deserialize;

    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;
    let mut total_diagnostics = Vec::new();
    if let Some(tombi_directives) = root.tombi_comment_directives() {
        let schema_store = schema_store().await;
        for TombiCommentDirective {
            content,
            content_range,
            ..
        } in tombi_directives
        {
            let (root, errors) =
                tombi_parser::parse(&content, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
                    .into_root_and_errors();
            // Check if there are any parsing errors
            if !errors.is_empty() {
                let mut diagnostics = Vec::new();
                for error in errors {
                    error.set_diagnostics(&mut diagnostics);
                }
                total_diagnostics.extend(
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, content_range)),
                );
                continue;
            }

            let (document_tree, errors) = root
                .into_document_tree_and_errors(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
                .into();

            // Check for errors during document tree construction
            if !errors.is_empty() {
                let mut diagnostics = Vec::new();
                for error in errors {
                    error.set_diagnostics(&mut diagnostics);
                }
                total_diagnostics.extend(
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, content_range)),
                );
            } else {
                let document_schema = root_comment_directive_document_schema().await;
                let schema_context = tombi_schema_store::SchemaContext {
                    toml_version: TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
                    root_schema: None,
                    sub_schema_url_map: None,
                    store: schema_store,
                };

                if let Err(diagnostics) = tombi_validator::validate(
                    document_tree.clone(),
                    root_comment_directive_source_schema(document_schema).await,
                    &schema_context,
                )
                .await
                {
                    total_diagnostics.extend(
                        diagnostics.into_iter().map(|diagnostic| {
                            into_directive_diagnostic(&diagnostic, content_range)
                        }),
                    );
                }
            }
            if let Some(total_document_tree_table) = total_document_tree_table.as_mut() {
                if let Err(errors) = total_document_tree_table.merge(document_tree.into()) {
                    let mut diagnostics = Vec::new();
                    for error in errors {
                        error.set_diagnostics(&mut diagnostics);
                    }
                    total_diagnostics.extend(
                        diagnostics.into_iter().map(|diagnostic| {
                            into_directive_diagnostic(&diagnostic, content_range)
                        }),
                    );
                }
            } else {
                total_document_tree_table = Some(document_tree.into());
            }
        }
    }

    if !total_diagnostics.is_empty() {
        return Err(total_diagnostics);
    }
    if let Some(total_document_tree_table) = total_document_tree_table {
        if let Ok(directive) = RootTombiDirective::deserialize(
            &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
        ) {
            return Ok(Some(directive));
        }
    }
    Ok(None)
}

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
static ROOT_COMMENT_DIRECTIVE_SCHEMA_URL: std::sync::OnceLock<SchemaUrl> =
    std::sync::OnceLock::new();
static ROOT_COMMENT_DIRECTIVE_SOURCE_SCHEMA: std::sync::OnceLock<SourceSchema> =
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

#[inline]
pub fn root_comment_directive_schema_url() -> &'static SchemaUrl {
    ROOT_COMMENT_DIRECTIVE_SCHEMA_URL.get_or_init(|| {
        let url = Url::parse("tombi://json.tombi.dev/root-tombi-directive.json").unwrap();
        SchemaUrl::new(url)
    })
}

pub async fn root_comment_directive_document_schema() -> DocumentSchema {
    let schema_store = schema_store().await;
    let schema_url = root_comment_directive_schema_url();
    let tombi_json::ValueNode::Object(object) = schema_store
        .fetch_schema_value(schema_url)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!(
            "Failed to fetch root comment directive schema from URL '{}'. \
             The fetched value was not an object.",
            schema_url
        );
    };
    DocumentSchema::new(object, schema_url.clone())
}

#[inline]
pub async fn root_comment_directive_source_schema(
    document_schema: DocumentSchema,
) -> &'static SourceSchema {
    ROOT_COMMENT_DIRECTIVE_SOURCE_SCHEMA.get_or_init(|| tombi_schema_store::SourceSchema {
        root_schema: Some(document_schema),
        sub_schema_url_map: AHashMap::with_capacity(0),
    })
}
