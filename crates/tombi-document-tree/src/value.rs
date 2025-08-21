mod array;
mod boolean;
mod date_time;
mod float;
mod integer;
mod string;
mod table;

pub use array::{Array, ArrayKind};
pub use boolean::Boolean;
pub use date_time::{LocalDate, LocalDateTime, LocalTime, OffsetDateTime};
pub use float::Float;
pub use integer::{Integer, IntegerKind};
pub use string::{String, StringKind};
pub use table::{Table, TableKind};
use tombi_ast::AstNode;

use crate::{support::comment::try_new_comment, DocumentTreeAndErrors, IntoDocumentTreeAndErrors};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(Boolean),
    Integer(Integer),
    Float(Float),
    String(String),
    OffsetDateTime(OffsetDateTime),
    LocalDateTime(LocalDateTime),
    LocalDate(LocalDate),
    LocalTime(LocalTime),
    Array(Array),
    Table(Table),
    Incomplete { range: tombi_text::Range },
}

impl Value {
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        match self {
            Value::Boolean(value) => value.range(),
            Value::Integer(value) => value.range(),
            Value::Float(value) => value.range(),
            Value::String(value) => value.range(),
            Value::OffsetDateTime(value) => value.range(),
            Value::LocalDateTime(value) => value.range(),
            Value::LocalDate(value) => value.range(),
            Value::LocalTime(value) => value.range(),
            Value::Array(value) => value.range(),
            Value::Table(value) => value.range(),
            Value::Incomplete { range } => *range,
        }
    }

    #[inline]
    pub fn symbol_range(&self) -> tombi_text::Range {
        match self {
            Value::Boolean(value) => value.symbol_range(),
            Value::Integer(value) => value.symbol_range(),
            Value::Float(value) => value.symbol_range(),
            Value::String(value) => value.symbol_range(),
            Value::OffsetDateTime(value) => value.symbol_range(),
            Value::LocalDateTime(value) => value.symbol_range(),
            Value::LocalDate(value) => value.symbol_range(),
            Value::LocalTime(value) => value.symbol_range(),
            Value::Array(value) => value.symbol_range(),
            Value::Table(value) => value.symbol_range(),
            Value::Incomplete { range } => *range,
        }
    }

    #[inline]
    pub fn leading_comments(&self) -> &[crate::Comment] {
        match self {
            Value::Boolean(boolean) => boolean.leading_comments(),
            Value::Integer(integer) => integer.leading_comments(),
            Value::Float(float) => float.leading_comments(),
            Value::String(string) => string.leading_comments(),
            Value::OffsetDateTime(offset_date_time) => offset_date_time.leading_comments(),
            Value::LocalDateTime(local_date_time) => local_date_time.leading_comments(),
            Value::LocalDate(local_date) => local_date.leading_comments(),
            Value::LocalTime(local_time) => local_time.leading_comments(),
            Value::Array(array) => array.leading_comments(),
            Value::Table(table) => table.leading_comments(),
            Value::Incomplete { .. } => &[],
        }
    }

    #[inline]
    pub fn trailing_comment(&self) -> Option<&crate::Comment> {
        match self {
            Value::Boolean(boolean) => boolean.trailing_comment(),
            Value::Integer(integer) => integer.trailing_comment(),
            Value::Float(float) => float.trailing_comment(),
            Value::String(string) => string.trailing_comment(),
            Value::OffsetDateTime(offset_date_time) => offset_date_time.trailing_comment(),
            Value::LocalDateTime(local_date_time) => local_date_time.trailing_comment(),
            Value::LocalDate(local_date) => local_date.trailing_comment(),
            Value::LocalTime(local_time) => local_time.trailing_comment(),
            Value::Array(array) => array.trailing_comment(),
            Value::Table(table) => table.trailing_comment(),
            Value::Incomplete { .. } => None,
        }
    }
}

impl crate::ValueImpl for Value {
    fn value_type(&self) -> crate::ValueType {
        match self {
            Value::Boolean(boolean) => boolean.value_type(),
            Value::Integer(integer) => integer.value_type(),
            Value::Float(float) => float.value_type(),
            Value::String(string) => string.value_type(),
            Value::OffsetDateTime(offset_date_time) => offset_date_time.value_type(),
            Value::LocalDateTime(local_date_time) => local_date_time.value_type(),
            Value::LocalDate(local_date) => local_date.value_type(),
            Value::LocalTime(local_time) => local_time.value_type(),
            Value::Array(array) => array.value_type(),
            Value::Table(table) => table.value_type(),
            Value::Incomplete { .. } => crate::ValueType::Incomplete,
        }
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::Value {
    fn into_document_tree_and_errors(
        self,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let mut errors = Vec::new();
        for comment in self.leading_comments() {
            if let Err(error) = try_new_comment(comment.as_ref()) {
                errors.push(error);
            }
        }

        if let Some(comment) = self.trailing_comment() {
            if let Err(error) = try_new_comment(comment.as_ref()) {
                errors.push(error);
            }
        }

        let mut document_tree_result = match self {
            tombi_ast::Value::BasicString(string) => {
                string.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::LiteralString(string) => {
                string.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::MultiLineBasicString(string) => {
                string.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::MultiLineLiteralString(string) => {
                string.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::IntegerBin(integer) => {
                integer.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::IntegerOct(integer) => {
                integer.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::IntegerDec(integer) => {
                integer.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::IntegerHex(integer) => {
                integer.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::Float(float) => float.into_document_tree_and_errors(toml_version),
            tombi_ast::Value::Boolean(boolean) => {
                boolean.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::Value::OffsetDateTime(dt) => dt.into_document_tree_and_errors(toml_version),
            tombi_ast::Value::LocalDateTime(dt) => dt.into_document_tree_and_errors(toml_version),
            tombi_ast::Value::LocalDate(date) => date.into_document_tree_and_errors(toml_version),
            tombi_ast::Value::LocalTime(time) => time.into_document_tree_and_errors(toml_version),
            tombi_ast::Value::Array(array) => array.into_document_tree_and_errors(toml_version),
            tombi_ast::Value::InlineTable(inline_table) => {
                inline_table.into_document_tree_and_errors(toml_version)
            }
        };

        errors.extend(document_tree_result.errors);
        document_tree_result.errors = errors;

        document_tree_result
    }
}
