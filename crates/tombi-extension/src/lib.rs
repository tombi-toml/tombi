mod completion;
mod definition;
mod document_link;
pub use completion::*;
pub use definition::*;
pub use document_link::*;

pub trait Extension {}

pub fn dig_accessors<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    accessors: &'a [tombi_schema_store::Accessor],
) -> Option<(
    &'a tombi_schema_store::Accessor,
    &'a tombi_document_tree::Value,
)> {
    if accessors.is_empty() {
        return None;
    }
    let first_key = accessors[0].as_key()?;
    let mut value = document_tree.get(first_key)?;
    let mut current_accessor = &accessors[0];
    for accessor in accessors[1..].iter() {
        match (accessor, value) {
            (tombi_schema_store::Accessor::Key(key), tombi_document_tree::Value::Table(table)) => {
                let next_value = table.get(key)?;
                current_accessor = accessor;
                value = next_value;
            }
            (
                tombi_schema_store::Accessor::Index(index),
                tombi_document_tree::Value::Array(array),
            ) => {
                let next_value = array.get(*index)?;
                current_accessor = accessor;
                value = next_value;
            }
            _ => return None,
        }
    }

    Some((current_accessor, value))
}

pub fn get_accessors(
    document_tree: &tombi_document_tree::DocumentTree,
    keys: &[tombi_document_tree::Key],
    position: tombi_text::Position,
) -> Vec<tombi_schema_store::Accessor> {
    let mut accessors = Vec::new();
    let mut current_value: &tombi_document_tree::Value = document_tree.into();

    for key in keys {
        current_value = find_value_in_current(current_value, key, &mut accessors, position);
        accessors.push(tombi_schema_store::Accessor::Key(key.value().to_string()));
    }

    if let tombi_document_tree::Value::Array(array) = current_value {
        for (index, value) in array.values().iter().enumerate() {
            if value.range().contains(position) {
                accessors.push(tombi_schema_store::Accessor::Index(index));
                break;
            }
        }
    }

    accessors
}

fn find_value_in_current<'a>(
    current_value: &'a tombi_document_tree::Value,
    key: &tombi_document_tree::Key,
    accessors: &mut Vec<tombi_schema_store::Accessor>,
    position: tombi_text::Position,
) -> &'a tombi_document_tree::Value {
    match current_value {
        tombi_document_tree::Value::Array(array) => {
            for (index, value) in array.values().iter().enumerate() {
                if value.range().contains(position) {
                    accessors.push(tombi_schema_store::Accessor::Index(index));
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
