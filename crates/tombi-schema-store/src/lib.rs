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
pub use options::Options;
pub use schema::*;
pub use store::SchemaStore;
pub use value_type::ValueType;

pub fn get_schema_name(schema_url: &SchemaUrl) -> Option<&str> {
    if let Some(path) = schema_url.path().split('/').last() {
        if !path.is_empty() {
            return Some(path);
        }
    }
    schema_url.host_str()
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
