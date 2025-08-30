mod error;
mod http_client;
pub mod json;
pub mod macros;
mod options;
mod schema;
mod store;
mod value_type;
mod x_taplo;

pub use error::Error;
pub use http_client::*;
use itertools::{Either, Itertools};
pub use options::Options;
pub use schema::*;
pub use store::SchemaStore;
pub use tombi_accessor::{Accessor, AccessorContext, AccessorKeyKind, Accessors, KeyContext};
pub use value_type::ValueType;

pub fn get_schema_name(schema_uri: &tombi_uri::Uri) -> Option<&str> {
    if let Some(path) = schema_uri.path().split('/').next_back() {
        if !path.is_empty() {
            return Some(path);
        }
    }
    schema_uri.host_str()
}

pub fn get_tombi_schemastore_content(schema_uri: &tombi_uri::Uri) -> Option<&'static str> {
    if schema_uri.scheme() != "tombi" {
        return None;
    }

    match schema_uri.host_str() {
        Some("json.schemastore.org") => match schema_uri.path() {
            "/api/json/catalog.json" => Some(include_str!(
                "../../../json.schemastore.org/api/json/catalog.json"
            )),
            "/cargo.json" => Some(include_str!("../../../json.schemastore.org/cargo.json")),
            "/pyproject.json" => Some(include_str!("../../../json.schemastore.org/pyproject.json")),
            "/tombi.json" => Some(include_str!("../../../json.schemastore.org/tombi.json")),
            _ => None,
        },
        Some("json.tombi.dev") => match schema_uri.path() {
            "/tombi-document-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-document-directive.json"
            )),
            "/boolean-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/boolean-key-value-tombi-directive.json"
            )),
            "/tombi-boolean-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-boolean-directive.json"
            )),
            "/integer-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/integer-key-value-tombi-directive.json"
            )),
            "/tombi-integer-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-integer-directive.json"
            )),
            "/float-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/float-key-value-tombi-directive.json"
            )),
            "/tombi-float-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-float-directive.json"
            )),
            "/string-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/string-key-value-tombi-directive.json"
            )),
            "/tombi-string-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-string-directive.json"
            )),
            "/offset-date-time-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/offset-date-time-key-value-tombi-directive.json"
            )),
            "/tombi-offset-date-time-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-offset-date-time-directive.json"
            )),
            "/local-date-time-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/local-date-time-key-value-tombi-directive.json"
            )),
            "/tombi-local-date-time-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-local-date-time-directive.json"
            )),
            "/local-date-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/local-date-key-value-tombi-directive.json"
            )),
            "/tombi-local-date-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-local-date-directive.json"
            )),
            "/local-time-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/local-time-key-value-tombi-directive.json"
            )),
            "/tombi-local-time-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-local-time-directive.json"
            )),
            "/array-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/array-key-value-tombi-directive.json"
            )),
            "/tombi-array-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-array-directive.json"
            )),
            "/table-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/table-key-value-tombi-directive.json"
            )),
            "/tombi-table-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/tombi-table-directive.json"
            )),
            "/root-table-key-value-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/root-table-key-value-tombi-directive.json"
            )),
            "/key-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/key-tombi-directive.json"
            )),
            _ => None,
        },

        // TODO: Remove this deprecated uri after v1.0.0 release.
        None => match schema_uri.path() {
            "/json/catalog.json" => Some(include_str!(
                "../../../json.schemastore.org/api/json/catalog.json"
            )),
            "/json/schemas/cargo.schema.json" => {
                Some(include_str!("../../../json.schemastore.org/cargo.json"))
            }
            "/json/schemas/pyproject.schema.json" => {
                Some(include_str!("../../../json.schemastore.org/pyproject.json"))
            }
            "/json/schemas/tombi.schema.json" => {
                Some(include_str!("../../../json.schemastore.org/tombi.json"))
            }
            _ => None,
        },
        _ => None,
    }
}

pub fn build_accessor_contexts(
    accessors: &[Accessor],
    key_contexts: &mut impl Iterator<Item = KeyContext>,
) -> Vec<AccessorContext> {
    accessors
        .iter()
        .filter_map(|accessor| match accessor {
            Accessor::Key(_) => Some(AccessorContext::Key(key_contexts.next()?)),
            Accessor::Index(_) => Some(AccessorContext::Index),
        })
        .collect_vec()
}

pub async fn lint_source_schema_from_ast(
    root: &tombi_ast::Root,
    source_uri_or_path: Option<Either<&tombi_uri::Uri, &std::path::Path>>,
    schema_store: &SchemaStore,
) -> (
    Option<SourceSchema>,
    Option<(crate::Error, tombi_text::Range)>,
) {
    match schema_store
        .resolve_source_schema_from_ast(root, source_uri_or_path)
        .await
    {
        Ok(Some(schema)) => (Some(schema), None),
        Ok(None) => (None, None),
        Err(error_with_range) => {
            let source_schema = if let Some(source_uri_or_path) = source_uri_or_path {
                schema_store
                    .resolve_source_schema(source_uri_or_path)
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            };
            (source_schema, Some(error_with_range))
        }
    }
}
