use std::str::FromStr;

use ahash::AHashMap;
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::{DocumentSchema, SchemaUri, SourceSchema};
use tombi_toml_version::TomlVersion;

use crate::{
    into_directive_diagnostic, schema_store, DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URI,
    DOCUMENT_COMMENT_DIRECTIVE_SOURCE_SCHEMA, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/document-tombi-directive.json")))]
pub struct TombiDocumentCommentDirective {
    /// # TOML version.
    ///
    /// This directive specifies the TOML version of this document, with the highest priority.
    #[cfg_attr(feature = "jsonschema", schemars(default = "TomlVersion::default"))]
    pub toml_version: Option<TomlVersion>,

    /// # Formatter options.
    pub format: Option<FormatOptions>,

    /// # Linter options.
    pub lint: Option<LintOptions>,

    /// # Schema options.
    pub schema: Option<SchemaOptions>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct FormatOptions {
    /// # Format disable
    ///
    /// Disable formatting for this document.
    #[cfg_attr(feature = "jsonschema", schemars(default = "default_false"))]
    pub disable: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LintOptions {
    /// # Lint disable
    ///
    /// Disable linting for this document.
    #[cfg_attr(feature = "jsonschema", schemars(default = "default_false"))]
    pub disable: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct SchemaOptions {
    /// # Enable strict schema validation.
    ///
    /// If `additionalProperties` is not specified in the JSON Schema,
    /// the strict mode treats it as `additionalProperties: false`,
    /// which is different from the JSON Schema specification.
    #[cfg_attr(feature = "jsonschema", schemars(default = "default_true"))]
    pub strict: Option<bool>,
}

#[cfg(feature = "jsonschema")]
#[allow(unused)]
#[inline]
fn default_true() -> Option<bool> {
    Some(true)
}

#[cfg(feature = "jsonschema")]
#[allow(unused)]
#[inline]
fn default_false() -> Option<bool> {
    Some(false)
}

pub async fn get_tombi_document_comment_directive(
    root: &tombi_ast::Root,
) -> Option<TombiDocumentCommentDirective> {
    get_tombi_document_comment_directive_and_diagnostics(root)
        .await
        .0
}

pub async fn get_tombi_document_comment_directive_and_diagnostics(
    root: &tombi_ast::Root,
) -> (
    Option<TombiDocumentCommentDirective>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    use serde::Deserialize;

    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;
    let mut total_diagnostics = Vec::new();
    if let Some(tombi_directives) = root.tombi_document_comment_directives() {
        let schema_store = schema_store().await;
        for tombi_ast::TombiDocumentCommentDirective {
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
                    sub_schema_uri_map: None,
                    store: schema_store,
                    strict: None,
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

    if let Some(total_document_tree_table) = total_document_tree_table {
        (
            TombiDocumentCommentDirective::deserialize(
                &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
            )
            .ok(),
            total_diagnostics,
        )
    } else {
        (None, total_diagnostics)
    }
}

#[inline]
pub fn document_comment_directive_schema_uri() -> &'static SchemaUri {
    DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URI.get_or_init(|| {
        let uri = tombi_uri::Uri::from_str("tombi://json.tombi.dev/document-tombi-directive.json")
            .unwrap();
        SchemaUri::new(uri)
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

#[inline]
pub async fn document_comment_directive_source_schema(
    document_schema: DocumentSchema,
) -> &'static SourceSchema {
    DOCUMENT_COMMENT_DIRECTIVE_SOURCE_SCHEMA.get_or_init(|| tombi_schema_store::SourceSchema {
        root_schema: Some(document_schema),
        sub_schema_uri_map: AHashMap::with_capacity(0),
    })
}
