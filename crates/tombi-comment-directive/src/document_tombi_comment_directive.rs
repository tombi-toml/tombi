use ahash::AHashMap;
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::{DocumentSchema, SchemaUrl, SourceSchema};
use tombi_toml_version::TomlVersion;
use url::Url;

use crate::{
    into_directive_diagnostic, schema_store, DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URL,
    DOCUMENT_COMMENT_DIRECTIVE_SOURCE_SCHEMA, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/document-tombi-directive.json")))]
pub struct DocumentTombiCommentDirective {
    /// # TOML version.
    ///
    /// This directive specifies the TOML version of this document, with the highest priority.
    pub toml_version: Option<TomlVersion>,
}

pub async fn get_document_tombi_comment_directive(
    root: &tombi_ast::Root,
) -> Option<DocumentTombiCommentDirective> {
    try_get_document_tombi_comment_directive(root)
        .await
        .ok()
        .flatten()
}

pub async fn try_get_document_tombi_comment_directive(
    root: &tombi_ast::Root,
) -> Result<Option<DocumentTombiCommentDirective>, Vec<tombi_diagnostic::Diagnostic>> {
    use serde::Deserialize;

    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;
    let mut total_diagnostics = Vec::new();
    if let Some(tombi_directives) = root.document_tombi_comment_directives() {
        let schema_store = schema_store().await;
        for tombi_ast::DocumentTombiCommentDirective {
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
                let document_schema = document_comment_directive_document_schema().await;
                let schema_context = tombi_schema_store::SchemaContext {
                    toml_version: TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
                    root_schema: None,
                    sub_schema_url_map: None,
                    store: schema_store,
                };

                if let Err(diagnostics) = tombi_validator::validate(
                    document_tree.clone(),
                    document_comment_directive_source_schema(document_schema).await,
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
        if let Ok(directive) = DocumentTombiCommentDirective::deserialize(
            &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
        ) {
            return Ok(Some(directive));
        }
    }
    Ok(None)
}

#[inline]
pub fn document_comment_directive_schema_url() -> &'static SchemaUrl {
    DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URL.get_or_init(|| {
        let url = Url::parse("tombi://json.tombi.dev/document-tombi-directive.json").unwrap();
        SchemaUrl::new(url)
    })
}

pub async fn document_comment_directive_document_schema() -> DocumentSchema {
    let schema_store = schema_store().await;
    let schema_url = document_comment_directive_schema_url();
    let tombi_json::ValueNode::Object(object) = schema_store
        .fetch_schema_value(schema_url)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!(
            "Failed to fetch document comment directive schema from URL '{}'. \
             The fetched value was not an object.",
            schema_url
        );
    };
    DocumentSchema::new(object, schema_url.clone())
}

#[inline]
pub async fn document_comment_directive_source_schema(
    document_schema: DocumentSchema,
) -> &'static SourceSchema {
    DOCUMENT_COMMENT_DIRECTIVE_SOURCE_SCHEMA.get_or_init(|| tombi_schema_store::SourceSchema {
        root_schema: Some(document_schema),
        sub_schema_url_map: AHashMap::with_capacity(0),
    })
}
