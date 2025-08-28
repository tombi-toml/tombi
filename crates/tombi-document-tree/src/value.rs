mod array;
mod boolean;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;
mod table;

pub use array::{Array, ArrayKind};
pub use boolean::Boolean;
pub use float::Float;
pub use integer::{Integer, IntegerKind};
pub use local_date::LocalDate;
pub use local_date_time::LocalDateTime;
pub use local_time::LocalTime;
pub use offset_date_time::OffsetDateTime;
pub use string::{String, StringKind};
pub use table::{Table, TableKind};
use tombi_ast::{AstNode, TombiValueCommentDirective};

use crate::{DocumentTreeAndErrors, IntoDocumentTreeAndErrors};

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
    pub fn comment_directives(&self) -> Option<&[TombiValueCommentDirective]> {
        match self {
            Value::Boolean(value) => value.comment_directives(),
            Value::Integer(value) => value.comment_directives(),
            Value::Float(value) => value.comment_directives(),
            Value::String(value) => value.comment_directives(),
            Value::OffsetDateTime(value) => value.comment_directives(),
            Value::LocalDateTime(value) => value.comment_directives(),
            Value::LocalDate(value) => value.comment_directives(),
            Value::LocalTime(value) => value.comment_directives(),
            Value::Array(value) => value.comment_directives(),
            Value::Table(value) => value.comment_directives(),
            Value::Incomplete { .. } => None,
        }
    }

    pub(crate) fn extend_comment_directives(
        &mut self,
        comment_directives: Vec<TombiValueCommentDirective>,
    ) {
        let value_comment_directives = match self {
            Value::Boolean(boolean) => &mut boolean.comment_directives,
            Value::Integer(integer) => &mut integer.comment_directives,
            Value::Float(float) => &mut float.comment_directives,
            Value::String(string) => &mut string.comment_directives,
            Value::OffsetDateTime(offset_date_time) => &mut offset_date_time.comment_directives,
            Value::LocalDateTime(local_date_time) => &mut local_date_time.comment_directives,
            Value::LocalDate(local_date) => &mut local_date.comment_directives,
            Value::LocalTime(local_time) => &mut local_time.comment_directives,
            Value::Array(array) => &mut array.comment_directives,
            Value::Table(table) => &mut table.comment_directives,
            Value::Incomplete { .. } => return,
        };

        if let Some(value_comment_directives) = value_comment_directives {
            value_comment_directives.extend(comment_directives);
        } else {
            *value_comment_directives = Some(Box::new(comment_directives));
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
        let mut comment_directives = vec![];

        for comment in self.leading_comments() {
            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                comment_directives.push(comment_directive);
            }
        }

        if let Some(comment) = self.trailing_comment() {
            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                comment_directives.push(comment_directive);
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

fn collect_comment_directives(node: impl AstNode) -> Option<Box<Vec<TombiValueCommentDirective>>> {
    let mut comment_directives = vec![];

    for comment in node.leading_comments() {
        if let Some(comment_directive) = comment.get_tombi_value_directive() {
            comment_directives.push(comment_directive);
        }
    }

    if let Some(comment) = node.trailing_comment() {
        if let Some(comment_directive) = comment.get_tombi_value_directive() {
            comment_directives.push(comment_directive);
        }
    }

    if !comment_directives.is_empty() {
        Some(Box::new(comment_directives))
    } else {
        None
    }
}
