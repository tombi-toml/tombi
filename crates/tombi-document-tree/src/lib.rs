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
/// NOTE: You cannot follow indices. Use `tombi_schema_store::dig_accessors` for that.
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
