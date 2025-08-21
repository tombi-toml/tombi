mod accessor;
mod error;
mod http_client;
pub mod json;
pub mod macros;
mod options;
mod schema;
mod store;
mod value_type;
mod x_taplo;

pub use accessor::{Accessor, Accessors};
pub use error::Error;
pub use http_client::*;
use itertools::{Either, Itertools};
pub use options::Options;
pub use schema::*;
pub use store::SchemaStore;
use tombi_ast::{algo::ancestors_at_position, AstNode};
use tombi_document_tree::TryIntoDocumentTree;
pub use value_type::ValueType;

pub use crate::accessor::{AccessorContext, AccessorKeyKind, KeyContext};

pub fn get_schema_name(schema_uri: &tombi_uri::Uri) -> Option<&str> {
    if let Some(path) = schema_uri.path().split('/').next_back() {
        if !path.is_empty() {
            return Some(path);
        }
    }
    schema_uri.host_str()
}

pub fn get_accessors(
    document_tree: &tombi_document_tree::DocumentTree,
    keys: &[tombi_document_tree::Key],
    position: tombi_text::Position,
) -> Vec<Accessor> {
    let mut accessors = Vec::new();
    let mut current_value: &tombi_document_tree::Value = document_tree.into();

    for key in keys {
        current_value = find_value_in_current(current_value, key, &mut accessors, position);
        accessors.push(Accessor::Key(key.value().to_string()));
    }

    if let tombi_document_tree::Value::Array(array) = current_value {
        for (index, value) in array.values().iter().enumerate() {
            if value.range().contains(position) {
                accessors.push(Accessor::Index(index));
                break;
            }
        }
    }

    accessors
}

fn find_value_in_current<'a>(
    current_value: &'a tombi_document_tree::Value,
    key: &tombi_document_tree::Key,
    accessors: &mut Vec<Accessor>,
    position: tombi_text::Position,
) -> &'a tombi_document_tree::Value {
    match current_value {
        tombi_document_tree::Value::Array(array) => {
            for (index, value) in array.values().iter().enumerate() {
                if value.range().contains(position) {
                    accessors.push(Accessor::Index(index));
                    return find_value_in_current(value, key, accessors, position);
                }
            }
        }
        tombi_document_tree::Value::Table(table) => {
            if let Some(value) = table.get(key) {
                return value;
            }
        }
        _ => {}
    }

    current_value
}

pub fn dig_accessors<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    accessors: &'a [crate::Accessor],
) -> Option<(&'a crate::Accessor, &'a tombi_document_tree::Value)> {
    if accessors.is_empty() {
        return None;
    }
    let first_key = accessors[0].as_key()?;
    let mut value = document_tree.get(first_key)?;
    let mut current_accessor = &accessors[0];
    for accessor in accessors[1..].iter() {
        match (accessor, value) {
            (crate::Accessor::Key(key), tombi_document_tree::Value::Table(table)) => {
                let next_value = table.get(key)?;
                current_accessor = accessor;
                value = next_value;
            }
            (crate::Accessor::Index(index), tombi_document_tree::Value::Array(array)) => {
                let next_value = array.get(*index)?;
                current_accessor = accessor;
                value = next_value;
            }
            _ => return None,
        }
    }

    Some((current_accessor, value))
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
            "/api/json/catalog.json" => Some(include_str!(
                "../../../json.tombi.dev/api/json/catalog.json"
            )),
            "/document-tombi-directive.json" => Some(include_str!(
                "../../../json.tombi.dev/document-tombi-directive.json"
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

pub async fn get_completion_keys_with_context(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    toml_version: tombi_config::TomlVersion,
) -> Option<(Vec<tombi_document_tree::Key>, Vec<KeyContext>)> {
    let mut keys_vec = vec![];
    let mut key_contexts = vec![];

    for node in ancestors_at_position(root.syntax(), position) {
        if let Some(kv) = tombi_ast::KeyValue::cast(node.to_owned()) {
            let keys = kv.keys()?;
            let keys = if keys.range().contains(position) {
                keys.keys()
                    .take_while(|key| key.token().unwrap().range().start <= position)
                    .collect_vec()
            } else {
                keys.keys().collect_vec()
            };
            for (i, key) in keys.into_iter().rev().enumerate() {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key_dt)) => {
                        let kind = if i == 0 {
                            AccessorKeyKind::KeyValue
                        } else {
                            AccessorKeyKind::Dotted
                        };
                        keys_vec.push(key_dt.clone());
                        key_contexts.push(KeyContext {
                            kind,
                            range: key_dt.range(),
                        });
                    }
                    _ => return None,
                }
            }
        } else if let Some(table) = tombi_ast::Table::cast(node.to_owned()) {
            if let Some(header) = table.header() {
                for key in header.keys().rev() {
                    match key.try_into_document_tree(toml_version) {
                        Ok(Some(key_dt)) => {
                            keys_vec.push(key_dt.clone());
                            key_contexts.push(KeyContext {
                                kind: AccessorKeyKind::Header,
                                range: key_dt.range(),
                            });
                        }
                        _ => return None,
                    }
                }
            }
        } else if let Some(array_of_table) = tombi_ast::ArrayOfTable::cast(node.to_owned()) {
            if let Some(header) = array_of_table.header() {
                for key in header.keys().rev() {
                    match key.try_into_document_tree(toml_version) {
                        Ok(Some(key_dt)) => {
                            keys_vec.push(key_dt.clone());
                            key_contexts.push(KeyContext {
                                kind: AccessorKeyKind::Header,
                                range: key_dt.range(),
                            });
                        }
                        _ => return None,
                    }
                }
            }
        }
    }

    if keys_vec.is_empty() {
        return None;
    }
    Some((
        keys_vec.into_iter().rev().collect(),
        key_contexts.into_iter().rev().collect(),
    ))
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
