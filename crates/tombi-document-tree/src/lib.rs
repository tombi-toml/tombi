mod error;
mod key;
mod literal_value;
mod root;
pub mod support;
mod value;
mod value_type;

pub use error::Error;
pub use key::{Key, KeyKind};
pub use literal_value::LiteralValueRef;
pub use root::DocumentTree;
use tombi_toml_version::TomlVersion;
pub use value::{
    Array, ArrayKind, Boolean, Float, Integer, IntegerKind, LocalDate, LocalDateTime, LocalTime,
    OffsetDateTime, String, StringKind, Table, TableKind, Value,
};
pub use value_type::ValueType;

/// A structure that holds an incomplete tree and errors that are the reason for the incompleteness.
///
/// [DocumentTree](crate::Root) needs to hold an incomplete tree and errors at the same time because it allows incomplete values.
/// If there are no errors, the tree is considered complete and can be converted to a [Document](tombi_document::Document).
pub struct DocumentTreeAndErrors<T> {
    pub tree: T,
    pub errors: Vec<crate::Error>,
}

impl<T> DocumentTreeAndErrors<T> {
    pub fn ok(self) -> Result<T, Vec<crate::Error>> {
        if self.errors.is_empty() {
            Ok(self.tree)
        } else {
            Err(self.errors)
        }
    }
}

impl<T> From<DocumentTreeAndErrors<T>> for (T, Vec<crate::Error>) {
    fn from(result: DocumentTreeAndErrors<T>) -> Self {
        (result.tree, result.errors)
    }
}

pub trait ValueImpl {
    fn value_type(&self) -> ValueType;

    fn range(&self) -> tombi_text::Range;
}

/// A structure that holds an incomplete tree and errors that are the reason for the incompleteness.
pub trait IntoDocumentTreeAndErrors<T> {
    fn into_document_tree_and_errors(self, toml_version: TomlVersion) -> DocumentTreeAndErrors<T>;
}

/// Get a complete tree or errors for incomplete reasons.
pub trait TryIntoDocumentTree<T> {
    fn try_into_document_tree(self, toml_version: TomlVersion) -> Result<T, Vec<crate::Error>>;
}

impl<T, U> TryIntoDocumentTree<T> for U
where
    U: IntoDocumentTreeAndErrors<T>,
{
    #[inline]
    fn try_into_document_tree(self, toml_version: TomlVersion) -> Result<T, Vec<crate::Error>> {
        self.into_document_tree_and_errors(toml_version).ok()
    }
}

/// Follows the given keys in order and retrieves the value if it exists.
///
/// NOTE: You cannot follow indices. Use `tombi_accessor::dig_accessors` for that.
pub fn dig_keys<'a, K>(
    document_tree: &'a crate::DocumentTree,
    keys: &[&K],
) -> Option<(&'a crate::Key, &'a crate::Value)>
where
    K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
{
    if keys.is_empty() {
        return None;
    }
    let (mut key, mut value) = document_tree.get_key_value(keys[0])?;
    for k in keys[1..].iter() {
        let crate::Value::Table(table) = value else {
            return None;
        };

        let (next_key, next_value) = table.get_key_value(*k)?;

        key = next_key;
        value = next_value;
    }

    Some((key, value))
}

pub fn dig_accessors<'a>(
    document_tree: &'a crate::DocumentTree,
    accessors: &'a [tombi_accessor::Accessor],
) -> Option<(&'a tombi_accessor::Accessor, &'a crate::Value)> {
    if accessors.is_empty() {
        return None;
    }
    let first_key = accessors[0].as_key()?;
    let mut value = document_tree.get(first_key)?;
    let mut current_accessor = &accessors[0];
    for accessor in accessors[1..].iter() {
        match (accessor, value) {
            (tombi_accessor::Accessor::Key(key), crate::Value::Table(table)) => {
                let next_value = table.get(key)?;
                current_accessor = accessor;
                value = next_value;
            }
            (tombi_accessor::Accessor::Index(index), crate::Value::Array(array)) => {
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
    document_tree: &crate::DocumentTree,
    keys: &[crate::Key],
    position: tombi_text::Position,
) -> Vec<tombi_accessor::Accessor> {
    let mut accessors = Vec::new();
    let mut current_value: &crate::Value = document_tree.into();

    for key in keys {
        current_value = find_value_in_current(current_value, key, &mut accessors, position);
        accessors.push(tombi_accessor::Accessor::Key(key.value.clone()));
    }

    if let crate::Value::Array(array) = current_value {
        for (index, value) in array.values().iter().enumerate() {
            if value.contains(position) {
                accessors.push(tombi_accessor::Accessor::Index(index));
                break;
            }
        }
    }

    accessors
}

fn find_value_in_current<'a>(
    current_value: &'a crate::Value,
    key: &crate::Key,
    accessors: &mut Vec<tombi_accessor::Accessor>,
    position: tombi_text::Position,
) -> &'a crate::Value {
    match current_value {
        crate::Value::Array(array) => {
            for (index, value) in array.values().iter().enumerate() {
                if value.contains(position) {
                    accessors.push(tombi_accessor::Accessor::Index(index));
                    return find_value_in_current(value, key, accessors, position);
                }
            }
        }
        crate::Value::Table(table) => {
            if let Some(value) = table.get(key) {
                return value;
            }
        }
        _ => {}
    }

    current_value
}
