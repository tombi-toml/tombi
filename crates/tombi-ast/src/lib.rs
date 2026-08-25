//! Stable, implementation-independent TOML AST operations.
//!
//! This crate deliberately contains no parser or tree representation. Tombi's
//! source-backed tape implements these traits in `tombi-ast-syntax`; extensions can
//! depend on this contract without depending on that representation.

use std::borrow::Cow;

use tombi_text::Range;
use tombi_toml_version::TomlVersion;

pub use tombi_date_time::{LocalDate, LocalDateTime, LocalTime, OffsetDateTime};

/// Common operations available on every public TOML AST value.
pub trait Node: Clone + std::fmt::Debug {
    /// The node's range in the parsed source.
    fn range(&self) -> Range;

    /// The exact, lossless source text covered by the node.
    fn text(&self) -> &str;
}

/// A directly matchable item in a TOML document root.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RootItem<'a, K, T, A> {
    KeyValue(&'a K),
    Table(&'a T),
    ArrayOfTable(&'a A),
}

/// The decoded semantic value represented by a TOML value node.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Value<'a, A, I> {
    String(Cow<'a, str>),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(&'a A),
    InlineTable(&'a I),
    OffsetDateTime(OffsetDateTime),
    LocalDateTime(LocalDateTime),
    LocalDate(LocalDate),
    LocalTime(LocalTime),
}

/// Operations on a TOML document root.
pub trait RootNode: Node {
    type Item: RootItemNode;
    type KeyValue: KeyValueNode;
    type Table: TableNode;
    type ArrayOfTable: ArrayOfTableNode;

    fn items(&self) -> impl Iterator<Item = Self::Item> + '_;
    fn key_values(&self) -> impl Iterator<Item = Self::KeyValue> + '_;
    fn tables(&self) -> impl Iterator<Item = Self::Table> + '_;
    fn array_of_tables(&self) -> impl Iterator<Item = Self::ArrayOfTable> + '_;
}

/// Operations on a root item node.
pub trait RootItemNode: Node {
    type KeyValue: KeyValueNode;
    type Table: TableNode;
    type ArrayOfTable: ArrayOfTableNode;

    fn item(&self) -> RootItem<'_, Self::KeyValue, Self::Table, Self::ArrayOfTable>;
}

/// Operations on a standard TOML table.
pub trait TableNode: Node {
    type Keys: KeysNode;
    type KeyValue: KeyValueNode;

    fn header(&self) -> Option<Self::Keys>;
    fn key_values(&self) -> impl Iterator<Item = Self::KeyValue> + '_;
}

/// Operations on a TOML array of tables.
pub trait ArrayOfTableNode: Node {
    type Keys: KeysNode;
    type KeyValue: KeyValueNode;

    fn header(&self) -> Option<Self::Keys>;
    fn key_values(&self) -> impl Iterator<Item = Self::KeyValue> + '_;
}

/// Operations on a TOML key-value pair.
pub trait KeyValueNode: Node {
    type Keys: KeysNode;
    type Value: ValueNode;

    fn keys(&self) -> Option<Self::Keys>;
    fn value(&self) -> Option<Self::Value>;
}

/// Operations on a dotted TOML key path.
pub trait KeysNode: Node {
    type Key: KeyNode;

    fn keys(&self) -> impl Iterator<Item = Self::Key> + '_;
}

/// Operations on one component of a TOML key path.
pub trait KeyNode: Node {
    /// Returns the semantic key without quotes or escape sequences.
    ///
    /// Unescaped text borrows from the source. Escaped text is decoded on demand.
    /// Invalid or version-incompatible keys return `None`.
    fn content(&self, toml_version: TomlVersion) -> Option<Cow<'_, str>>;
}

/// Operations on a TOML value node.
pub trait ValueNode: Node {
    type Array: ArrayNode;
    type InlineTable: InlineTableNode;

    fn value(&self, toml_version: TomlVersion)
    -> Option<Value<'_, Self::Array, Self::InlineTable>>;
}

/// Operations on a TOML array.
pub trait ArrayNode: Node {
    type Value: ValueNode;

    fn values(&self) -> impl Iterator<Item = Self::Value> + '_;
}

/// Operations on a TOML inline table.
pub trait InlineTableNode: Node {
    type KeyValue: KeyValueNode;

    fn key_values(&self) -> impl Iterator<Item = Self::KeyValue> + '_;
}

/// Operations exposed for comments passed through the extension API.
pub trait CommentNode: Node {}
