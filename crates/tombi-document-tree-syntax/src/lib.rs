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
pub use tombi_ast_syntax::DecodedTextResolver;
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

    fn span(&self) -> tombi_text::Span;
}

pub trait LikeString {
    fn value(&self) -> &str;

    fn comment_directives(&self) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_>;
}

/// A structure that holds an incomplete tree and errors that are the reason for the incompleteness.
///
/// `decoded` is the pool of escaped strings made by [`tombi_ast_syntax::AstNode::decode_strings`].
/// The tree borrows from it, so the caller keeps it next to the tree.
pub trait IntoDocumentTreeAndErrors<'t, T> {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
        decoded: &'t DecodedTextResolver,
    ) -> DocumentTreeAndErrors<T>;
}

pub(crate) struct DocumentTreeContext<'t> {
    toml_version: TomlVersion,
    decoded_text: &'t DecodedTextResolver,
}

impl<'t> DocumentTreeContext<'t> {
    pub(crate) fn new(toml_version: TomlVersion, decoded_text: &'t DecodedTextResolver) -> Self {
        Self {
            toml_version,
            decoded_text,
        }
    }
}

pub(crate) trait IntoDocumentTreeWithContext<'t, T> {
    fn into_document_tree_with_context(
        self,
        context: &DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<T>;
}

macro_rules! impl_into_document_tree_and_errors {
    ($syntax:ident => $tree:ty) => {
        impl<'t> IntoDocumentTreeAndErrors<'t, $tree> for tombi_ast_syntax::$syntax<'t> {
            fn into_document_tree_and_errors(
                self,
                toml_version: TomlVersion,
                decoded: &'t DecodedTextResolver,
            ) -> DocumentTreeAndErrors<$tree> {
                let context = DocumentTreeContext::new(toml_version, decoded);
                self.into_document_tree_with_context(&context)
            }
        }
    };
}

impl_into_document_tree_and_errors!(Root => DocumentTree<'t>);
impl_into_document_tree_and_errors!(Key => Option<Key<'t>>);
impl_into_document_tree_and_errors!(Keys => Vec<Key<'t>>);
impl_into_document_tree_and_errors!(Value => Value<'t>);
impl_into_document_tree_and_errors!(Boolean => Value<'t>);
impl_into_document_tree_and_errors!(Float => Value<'t>);
impl_into_document_tree_and_errors!(IntegerBin => Value<'t>);
impl_into_document_tree_and_errors!(IntegerOct => Value<'t>);
impl_into_document_tree_and_errors!(IntegerDec => Value<'t>);
impl_into_document_tree_and_errors!(IntegerHex => Value<'t>);
impl_into_document_tree_and_errors!(LocalDate => Value<'t>);
impl_into_document_tree_and_errors!(LocalTime => Value<'t>);
impl_into_document_tree_and_errors!(LocalDateTime => Value<'t>);
impl_into_document_tree_and_errors!(OffsetDateTime => Value<'t>);
impl_into_document_tree_and_errors!(BasicString => Value<'t>);
impl_into_document_tree_and_errors!(LiteralString => Value<'t>);
impl_into_document_tree_and_errors!(MultiLineBasicString => Value<'t>);
impl_into_document_tree_and_errors!(MultiLineLiteralString => Value<'t>);
impl_into_document_tree_and_errors!(Array => Value<'t>);
impl_into_document_tree_and_errors!(InlineTable => Value<'t>);
impl_into_document_tree_and_errors!(Table => Table<'t>);
impl_into_document_tree_and_errors!(ArrayOfTable => Table<'t>);
impl_into_document_tree_and_errors!(TableOrArrayOfTable => Table<'t>);
impl_into_document_tree_and_errors!(KeyValue => Table<'t>);

/// Get a complete tree or errors for incomplete reasons.
pub trait TryIntoDocumentTree<'t, T> {
    fn try_into_document_tree(
        self,
        toml_version: TomlVersion,
        decoded: &'t DecodedTextResolver,
    ) -> Result<T, Vec<crate::Error>>;
}

impl<'t, T, U> TryIntoDocumentTree<'t, T> for U
where
    U: IntoDocumentTreeAndErrors<'t, T>,
{
    #[inline]
    fn try_into_document_tree(
        self,
        toml_version: TomlVersion,
        decoded: &'t DecodedTextResolver,
    ) -> Result<T, Vec<crate::Error>> {
        self.into_document_tree_and_errors(toml_version, decoded)
            .ok()
    }
}

/// Follows the given keys in order and retrieves the value if it exists.
///
/// NOTE: You cannot follow indices. Use [`dig_accessors`] for that.
pub fn dig_keys<'a, 't, K>(
    table: &'a crate::Table<'t>,
    keys: &[&K],
) -> Option<(&'a crate::Key<'t>, &'a crate::Value<'t>)>
where
    K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key<'t>>,
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

pub fn get_accessors<'t>(
    document_tree: &crate::DocumentTree<'t>,
    keys: &[crate::Key<'t>],
    offset: tombi_text::Offset,
) -> Vec<tombi_accessor::Accessor> {
    let mut accessors = Vec::new();
    let mut current = CurrentValue::Root(document_tree);

    for key in keys {
        current = find_value_in_current(current, key, &mut accessors, offset);
        accessors.push(tombi_accessor::Accessor::Key(key.value().to_owned()));
    }

    if let CurrentValue::Value(crate::Value::Array(array)) = current {
        for (index, value) in array.values().iter().enumerate() {
            if value.contains(offset) {
                accessors.push(tombi_accessor::Accessor::Index(index));
                break;
            }
        }
    }

    accessors
}

#[derive(Clone, Copy)]
enum CurrentValue<'a, 't> {
    Root(&'a crate::Table<'t>),
    Value(&'a crate::Value<'t>),
}

fn find_value_in_current<'a, 't>(
    current: CurrentValue<'a, 't>,
    key: &crate::Key<'t>,
    accessors: &mut Vec<tombi_accessor::Accessor>,
    offset: tombi_text::Offset,
) -> CurrentValue<'a, 't> {
    match current {
        CurrentValue::Root(table) => table.get(key).map_or(current, CurrentValue::Value),
        CurrentValue::Value(crate::Value::Array(array)) => {
            for (index, value) in array.values().iter().enumerate() {
                if value.contains(offset) {
                    accessors.push(tombi_accessor::Accessor::Index(index));
                    return find_value_in_current(
                        CurrentValue::Value(value),
                        key,
                        accessors,
                        offset,
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
