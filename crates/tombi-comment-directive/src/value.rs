mod array;
mod boolean;
mod float;
mod integer;
mod key;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;
mod table;

pub use array::*;
pub use boolean::*;
pub use float::*;
pub use integer::*;
pub use key::*;
pub use local_date::*;
pub use local_date_time::*;
pub use local_time::*;
pub use offset_date_time::*;
use serde::Deserialize;
pub use string::*;
pub use table::*;

use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::{DocumentSchema, SchemaUri};
use tombi_severity_level::SeverityLevelDefaultError;

use crate::{
    into_directive_diagnostic, schema_store, source_schema, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(
    bound = "T: serde::de::DeserializeOwned + serde::Serialize + ValueTombiCommentDirectiveImpl"
)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct ValueTombiCommentDirective<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    lint: Option<ValueLintOptions<T>>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "T: serde::de::DeserializeOwned + serde::Serialize ")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct ValueLintOptions<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    rules: Option<T>,
}

pub trait ValueTombiCommentDirectiveImpl {
    fn value_comment_directive_schema_url() -> SchemaUri;
}

/// Common validation settings for all value types
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct CommonValueTombiCommentDirectiveRules {
    /// Controls the severity level for type mismatch errors
    pub type_mismatch: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for const value errors
    pub const_value: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for enumerate value errors
    pub enumerate: Option<SeverityLevelDefaultError>,
}

pub async fn get_tombi_value_comment_directive<
    T: serde::de::DeserializeOwned + serde::Serialize + ValueTombiCommentDirectiveImpl,
>(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
) -> Option<ValueTombiCommentDirective<T>> {
    get_tombi_value_comment_directive_and_diagnostics(comment_directives)
        .await
        .0
}

pub async fn get_tombi_value_comment_directive_and_diagnostics<
    T: serde::de::DeserializeOwned + serde::Serialize + ValueTombiCommentDirectiveImpl,
>(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
) -> (
    Option<ValueTombiCommentDirective<T>>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;
    let mut total_diagnostics = Vec::new();
    let schema_store = schema_store().await;

    for tombi_ast::TombiValueCommentDirective {
        content,
        content_range,
        ..
    } in comment_directives
    {
        let (root, errors) = tombi_parser::parse(&content, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
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
                    .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
            );
            continue;
        }

        let (document_tree, errors) = root
            .into_document_tree_and_errors(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
            .into();

        if !errors.is_empty() {
            let mut diagnostics = Vec::new();
            for error in errors {
                error.set_diagnostics(&mut diagnostics);
            }
            total_diagnostics.extend(
                diagnostics
                    .into_iter()
                    .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
            );
        } else {
            let schema_uri = T::value_comment_directive_schema_url();
            let tombi_json::ValueNode::Object(object) = schema_store
                .fetch_schema_value(&schema_uri)
                .await
                .unwrap()
                .unwrap()
            else {
                panic!(
                    "Failed to fetch value comment directive schema from URL '{schema_uri}'. \
                    The fetched value was not an object."
                );
            };
            let document_schema = DocumentSchema::new(object, schema_uri.clone());

            let schema_context = tombi_schema_store::SchemaContext {
                toml_version: TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
                root_schema: None,
                sub_schema_uri_map: None,
                store: schema_store,
                strict: None,
            };

            if let Err(diagnostics) = tombi_validator::validate(
                document_tree.clone(),
                source_schema(document_schema).await,
                &schema_context,
            )
            .await
            {
                total_diagnostics.extend(
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
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
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
                );
            }
        } else {
            total_document_tree_table = Some(document_tree.into());
        }
    }

    if let Some(total_document_tree_table) = total_document_tree_table {
        (
            ValueTombiCommentDirective::<T>::deserialize(
                &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
            )
            .ok(),
            total_diagnostics,
        )
    } else {
        (None, total_diagnostics)
    }
}
