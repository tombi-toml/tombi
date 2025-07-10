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
use itertools::Itertools;
pub use options::Options;
pub use schema::*;
pub use store::SchemaStore;
use tombi_ast::{algo::ancestors_at_position, AstNode};
use tombi_document_tree::TryIntoDocumentTree;
use url::Url;
pub use value_type::ValueType;

pub use crate::accessor::{AccessorContext, AccessorKeyKind, KeyContext};

pub fn get_schema_name(schema_url: &Url) -> Option<&str> {
    if let Some(path) = schema_url.path().split('/').next_back() {
        if !path.is_empty() {
            return Some(path);
        }
    }
    schema_url.host_str()
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

pub fn get_tombi_scheme_content(schema_url: &url::Url) -> Option<&'static str> {
    match schema_url.path() {
        "/json/catalog.json" => Some(include_str!("../../../schemas/catalog.json")),
        "/json/schemas/cargo.schema.json" => {
            Some(include_str!("../../../schemas/cargo.schema.json"))
        }
        "/json/schemas/pyproject.schema.json" => {
            Some(include_str!("../../../schemas/pyproject.schema.json"))
        }
        "/json/schemas/tombi.schema.json" => {
            Some(include_str!("../../../schemas/tombi.schema.json"))
        }
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
