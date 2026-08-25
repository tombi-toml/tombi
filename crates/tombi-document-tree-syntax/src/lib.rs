mod api;
mod error;
mod key;
mod literal_value;
mod root;
mod support;
mod text;
mod value;
mod value_type;

pub use error::Error;
pub use key::Key;
pub use literal_value::LiteralValueRef;
pub use root::DocumentTree;
pub use text::DocumentText;
use tombi_ast_syntax::TombiValueCommentDirective;
pub use tombi_document_tree::{
    ArrayKind, IntegerKind, KeyKind, StringKind, TableKind, dig_accessors,
};
use tombi_toml_version::TomlVersion;
pub use value::{
    Array, Boolean, Float, Integer, LocalDate, LocalDateTime, LocalTime, OffsetDateTime, String,
    Table, Value,
};
pub use value_type::ValueType;

/// A structure that holds an incomplete tree and errors that are the reason for the incompleteness.
///
/// [`DocumentTree`] needs to hold an incomplete tree and errors at the same time because it allows incomplete values.
/// If there are no errors, the tree is considered complete and can be converted to an owned document.
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

pub trait LikeString {
    fn value(&self) -> &str;

    fn comment_directives(&self) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_>;
}

/// A structure that holds an incomplete tree and errors that are the reason for the incompleteness.
pub trait IntoDocumentTreeAndErrors<T> {
    fn into_document_tree_and_errors(self, toml_version: TomlVersion) -> DocumentTreeAndErrors<T>;
}

pub(crate) struct DocumentTreeContext {
    toml_version: TomlVersion,
    decoded_text: tombi_ast_syntax::DecodedTextResolver,
}

impl DocumentTreeContext {
    pub(crate) fn new(node: &tombi_ast_syntax::SyntaxNode, toml_version: TomlVersion) -> Self {
        Self {
            toml_version,
            decoded_text: node.decoded_text_resolver(toml_version),
        }
    }
}

pub(crate) trait IntoDocumentTreeWithContext<T> {
    fn into_document_tree_with_context(
        self,
        context: &DocumentTreeContext,
    ) -> DocumentTreeAndErrors<T>;
}

macro_rules! impl_into_document_tree_and_errors {
    ($syntax:ty => $tree:ty) => {
        impl IntoDocumentTreeAndErrors<$tree> for $syntax {
            fn into_document_tree_and_errors(
                self,
                toml_version: TomlVersion,
            ) -> DocumentTreeAndErrors<$tree> {
                let context = DocumentTreeContext::new(
                    tombi_ast_syntax::AstNode::syntax(&self),
                    toml_version,
                );
                self.into_document_tree_with_context(&context)
            }
        }
    };
}

impl_into_document_tree_and_errors!(tombi_ast_syntax::Root => DocumentTree);
impl_into_document_tree_and_errors!(tombi_ast_syntax::Key => Option<Key>);
impl_into_document_tree_and_errors!(tombi_ast_syntax::Keys => Vec<Key>);
impl_into_document_tree_and_errors!(tombi_ast_syntax::Value => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::Boolean => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::Float => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::IntegerBin => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::IntegerOct => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::IntegerDec => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::IntegerHex => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::LocalDate => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::LocalTime => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::LocalDateTime => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::OffsetDateTime => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::BasicString => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::LiteralString => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::MultiLineBasicString => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::MultiLineLiteralString => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::Array => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::InlineTable => Value);
impl_into_document_tree_and_errors!(tombi_ast_syntax::Table => Table);
impl_into_document_tree_and_errors!(tombi_ast_syntax::ArrayOfTable => Table);
impl_into_document_tree_and_errors!(tombi_ast_syntax::TableOrArrayOfTable => Table);
impl_into_document_tree_and_errors!(tombi_ast_syntax::KeyValue => Table);

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
/// NOTE: You cannot follow indices. Use [`dig_accessors`] for that.
pub fn dig_keys<'a, K>(
    table: &'a crate::Table,
    keys: &[&K],
) -> Option<(&'a crate::Key, &'a crate::Value)>
where
    K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key>,
{
    if keys.is_empty() {
        return None;
    }
    let (mut key, mut value) = table.get_key_value(keys[0])?;
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

pub fn get_accessors(
    document_tree: &crate::DocumentTree,
    keys: &[crate::Key],
    position: tombi_text::Position,
) -> Vec<tombi_accessor::Accessor> {
    let mut accessors = Vec::new();
    let mut current = CurrentValue::Root(document_tree);

    for key in keys {
        current = find_value_in_current(current, key, &mut accessors, position);
        accessors.push(tombi_accessor::Accessor::Key(key.value().to_owned()));
    }

    if let CurrentValue::Value(crate::Value::Array(array)) = current {
        for (index, value) in array.values().iter().enumerate() {
            if value.contains(position) {
                accessors.push(tombi_accessor::Accessor::Index(index));
                break;
            }
        }
    }

    accessors
}

#[derive(Clone, Copy)]
enum CurrentValue<'a> {
    Root(&'a crate::Table),
    Value(&'a crate::Value),
}

fn find_value_in_current<'a>(
    current: CurrentValue<'a>,
    key: &crate::Key,
    accessors: &mut Vec<tombi_accessor::Accessor>,
    position: tombi_text::Position,
) -> CurrentValue<'a> {
    match current {
        CurrentValue::Root(table) => table.get(key).map_or(current, CurrentValue::Value),
        CurrentValue::Value(crate::Value::Array(array)) => {
            for (index, value) in array.values().iter().enumerate() {
                if value.contains(position) {
                    accessors.push(tombi_accessor::Accessor::Index(index));
                    return find_value_in_current(
                        CurrentValue::Value(value),
                        key,
                        accessors,
                        position,
                    );
                }
            }
            current
        }
        CurrentValue::Value(crate::Value::Table(table)) => {
            table.get(key).map_or(current, CurrentValue::Value)
        }
        CurrentValue::Value(_) => current,
    }
}
