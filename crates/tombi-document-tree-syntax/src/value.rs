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

pub use array::Array;
pub use boolean::Boolean;
pub use float::Float;
pub use integer::Integer;
pub use local_date::LocalDate;
pub use local_date_time::LocalDateTime;
pub use local_time::LocalTime;
pub use offset_date_time::OffsetDateTime;
pub use string::String;
pub use table::Table;
use tombi_ast_syntax::{AstNode, TombiValueCommentDirective};
use tombi_document_tree::{ArrayKind, TableKind};

use crate::{DocumentTreeAndErrors, IntoDocumentTreeWithContext};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'t> {
    Boolean(Boolean),
    Integer(Integer),
    Float(Float),
    String(String<'t>),
    OffsetDateTime(OffsetDateTime),
    LocalDateTime(LocalDateTime),
    LocalDate(LocalDate),
    LocalTime(LocalTime),
    Array(Array<'t>),
    Table(Table<'t>),
    Incomplete { span: tombi_text::Span },
}

impl<'t> Value<'t> {
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        match self {
            Value::Boolean(value) => value.span(),
            Value::Integer(value) => value.span(),
            Value::Float(value) => value.span(),
            Value::String(value) => value.span(),
            Value::OffsetDateTime(value) => value.span(),
            Value::LocalDateTime(value) => value.span(),
            Value::LocalDate(value) => value.span(),
            Value::LocalTime(value) => value.span(),
            Value::Array(value) => value.span(),
            Value::Table(value) => value.span(),
            Value::Incomplete { span } => *span,
        }
    }

    #[inline]
    pub fn symbol_span(&self) -> tombi_text::Span {
        match self {
            Value::Boolean(value) => value.span(),
            Value::Integer(value) => value.span(),
            Value::Float(value) => value.span(),
            Value::String(value) => value.span(),
            Value::OffsetDateTime(value) => value.span(),
            Value::LocalDateTime(value) => value.span(),
            Value::LocalDate(value) => value.span(),
            Value::LocalTime(value) => value.span(),
            Value::Array(value) => value.symbol_span(),
            Value::Table(value) => value.symbol_span(),
            Value::Incomplete { span } => *span,
        }
    }

    #[inline]
    pub fn comment_directives(
        &self,
    ) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        match self {
            Value::Boolean(value) => value.comment_directives.as_deref(),
            Value::Integer(value) => value.comment_directives.as_deref(),
            Value::Float(value) => value.comment_directives.as_deref(),
            Value::String(value) => value.comment_directives.as_deref(),
            Value::OffsetDateTime(value) => value.comment_directives.as_deref(),
            Value::LocalDateTime(value) => value.comment_directives.as_deref(),
            Value::LocalDate(value) => value.comment_directives.as_deref(),
            Value::LocalTime(value) => value.comment_directives.as_deref(),
            Value::Array(value) => value.header_comment_directives.as_deref(),
            Value::Table(value) => value.header_comment_directives.as_deref(),
            Value::Incomplete { .. } => None,
        }
        .map(|s| s.iter())
    }

    pub fn is_inline(&self) -> bool {
        match self {
            Value::Boolean(_)
            | Value::Integer(_)
            | Value::Float(_)
            | Value::String(_)
            | Value::OffsetDateTime(_)
            | Value::LocalDateTime(_)
            | Value::LocalDate(_)
            | Value::LocalTime(_) => true,
            Value::Array(array) if array.kind() == ArrayKind::Array => true,
            Value::Table(table) if matches!(table.kind(), TableKind::InlineTable { .. }) => true,
            Value::Array(_) | Value::Table(_) | Value::Incomplete { .. } => false,
        }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Value::Boolean(_)
                | Value::Integer(_)
                | Value::Float(_)
                | Value::String(_)
                | Value::OffsetDateTime(_)
                | Value::LocalDateTime(_)
                | Value::LocalDate(_)
                | Value::LocalTime(_)
                | Value::Incomplete { .. }
        )
    }

    pub(crate) fn set_comment_directives(
        &mut self,
        comment_directives: Vec<TombiValueCommentDirective>,
    ) {
        match self {
            Value::Boolean(boolean) => boolean.comment_directives = Some(comment_directives),
            Value::Integer(integer) => integer.comment_directives = Some(comment_directives),
            Value::Float(float) => float.comment_directives = Some(comment_directives),
            Value::String(string) => string.comment_directives = Some(comment_directives),
            Value::OffsetDateTime(offset_date_time) => {
                offset_date_time.comment_directives = Some(comment_directives)
            }
            Value::LocalDateTime(local_date_time) => {
                local_date_time.comment_directives = Some(comment_directives)
            }
            Value::LocalDate(local_date) => {
                local_date.comment_directives = Some(comment_directives)
            }
            Value::LocalTime(local_time) => {
                local_time.comment_directives = Some(comment_directives)
            }
            Value::Array(array) => array.header_comment_directives = Some(comment_directives),
            Value::Table(table) => table.header_comment_directives = Some(comment_directives),
            Value::Incomplete { .. } => (),
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
            Value::Array(array) => &mut array.header_comment_directives,
            Value::Table(table) => &mut table.header_comment_directives,
            Value::Incomplete { .. } => return,
        };

        if let Some(value_comment_directives) = value_comment_directives {
            value_comment_directives.extend(comment_directives);
        } else {
            *value_comment_directives = Some(comment_directives);
        }
    }

    pub fn contains(&self, offset: tombi_text::Offset) -> bool {
        self.span().contains_inclusive(offset)
            || self.comment_directives().is_some_and(|mut directives| {
                directives
                    .any(|comment_directive| comment_directive.span().contains_inclusive(offset))
            })
    }
}

impl<'t> std::fmt::Display for Value<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Integer(integer) => write!(f, "{}", integer),
            Value::Float(float) => write!(f, "{}", float),
            Value::String(string) => write!(f, "{}", string),
            Value::OffsetDateTime(offset_date_time) => write!(f, "{}", offset_date_time),
            Value::LocalDateTime(local_date_time) => write!(f, "{}", local_date_time),
            Value::LocalDate(local_date) => write!(f, "{}", local_date),
            Value::LocalTime(local_time) => write!(f, "{}", local_time),
            Value::Array(array) => write!(f, "{}", array),
            Value::Table(table) => write!(f, "{}", table),
            Value::Incomplete { .. } => write!(f, "null"),
        }
    }
}

impl<'t> crate::ValueImpl for Value<'t> {
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

    fn span(&self) -> tombi_text::Span {
        self.span()
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::Value<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let mut errors = Vec::new();
        let mut comment_directives = vec![];

        for comment in self.leading_comments() {
            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                comment_directives.push(comment_directive);
            }
        }

        if let Some(comment) = self.trailing_comment()
            && let Some(comment_directive) = comment.get_tombi_value_directive()
        {
            comment_directives.push(comment_directive);
        }

        let mut document_tree_result = match self {
            tombi_ast_syntax::Value::BasicString(string) => {
                string.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::LiteralString(string) => {
                string.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::MultiLineBasicString(string) => {
                string.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::MultiLineLiteralString(string) => {
                string.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::IntegerBin(integer) => {
                integer.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::IntegerOct(integer) => {
                integer.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::IntegerDec(integer) => {
                integer.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::IntegerHex(integer) => {
                integer.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::Float(float) => float.into_document_tree_with_context(context),
            tombi_ast_syntax::Value::Boolean(boolean) => {
                boolean.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::OffsetDateTime(dt) => {
                dt.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::LocalDateTime(dt) => {
                dt.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::LocalDate(date) => {
                date.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::LocalTime(time) => {
                time.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::Value::Array(array) => array.into_document_tree_with_context(context),
            tombi_ast_syntax::Value::InlineTable(inline_table) => {
                inline_table.into_document_tree_with_context(context)
            }
        };

        errors.extend(document_tree_result.errors);
        document_tree_result.errors = errors;

        document_tree_result
    }
}

fn collect_comment_directives_and_errors<'t>(
    node: &impl AstNode<'t>,
) -> (Option<Vec<TombiValueCommentDirective>>, Vec<crate::Error>) {
    let mut comment_directives = vec![];
    let mut errors = vec![];

    for comment in node.leading_comments() {
        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
            errors.push(error);
        }

        if let Some(comment_directive) = comment.get_tombi_value_directive() {
            comment_directives.push(comment_directive);
        }
    }

    if let Some(comment) = node.trailing_comment() {
        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
            errors.push(error);
        }

        if let Some(comment_directive) = comment.get_tombi_value_directive() {
            comment_directives.push(comment_directive);
        }
    }

    if !comment_directives.is_empty() {
        (Some(comment_directives), errors)
    } else {
        (None, errors)
    }
}
