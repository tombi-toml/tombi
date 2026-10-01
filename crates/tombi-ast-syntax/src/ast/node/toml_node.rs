use crate::{
    Array, ArrayOfTable, AstNode, BasicString, Boolean, Float, InlineTable, IntegerBin, IntegerDec,
    IntegerHex, IntegerOct, KeyValue, Keys, LiteralString, LocalDate, LocalDateTime, LocalTime,
    MultiLineBasicString, MultiLineLiteralString, OffsetDateTime, Root, SyntaxKind, SyntaxNode,
    Table,
};

/// A closed, TOML-specific view of a node in source order.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TomlNode<'t> {
    Root(Root<'t>),
    Table(Table<'t>),
    ArrayOfTable(ArrayOfTable<'t>),
    KeyValue(KeyValue<'t>),
    Keys(Keys<'t>),
    Array(Array<'t>),
    InlineTable(InlineTable<'t>),
    BasicString(BasicString<'t>),
    Boolean(Boolean<'t>),
    Float(Float<'t>),
    IntegerBin(IntegerBin<'t>),
    IntegerDec(IntegerDec<'t>),
    IntegerHex(IntegerHex<'t>),
    IntegerOct(IntegerOct<'t>),
    LiteralString(LiteralString<'t>),
    LocalDate(LocalDate<'t>),
    LocalDateTime(LocalDateTime<'t>),
    LocalTime(LocalTime<'t>),
    MultiLineBasicString(MultiLineBasicString<'t>),
    MultiLineLiteralString(MultiLineLiteralString<'t>),
    OffsetDateTime(OffsetDateTime<'t>),
    Invalid(tombi_text::Span),
}

/// Commas adjacent to the syntax item at a cursor offset.
///
/// This includes commas recovered inside invalid syntax while the user is
/// typing an incomplete array or inline table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdjacentCommas {
    pub before: Option<tombi_text::Span>,
    pub after: Option<tombi_text::Span>,
}

impl<'t> TomlNode<'t> {
    pub(crate) fn cast(node: SyntaxNode<'t>) -> Option<Self> {
        Some(match node.kind() {
            SyntaxKind::ROOT => Self::Root(Root::cast(node)?),
            SyntaxKind::TABLE => Self::Table(Table::cast(node)?),
            SyntaxKind::ARRAY_OF_TABLE => Self::ArrayOfTable(ArrayOfTable::cast(node)?),
            SyntaxKind::KEY_VALUE => Self::KeyValue(KeyValue::cast(node)?),
            SyntaxKind::KEYS => Self::Keys(Keys::cast(node)?),
            SyntaxKind::ARRAY => Self::Array(Array::cast(node)?),
            SyntaxKind::INLINE_TABLE => Self::InlineTable(InlineTable::cast(node)?),
            SyntaxKind::BASIC_STRING => Self::BasicString(BasicString::cast(node)?),
            SyntaxKind::BOOLEAN => Self::Boolean(Boolean::cast(node)?),
            SyntaxKind::FLOAT => Self::Float(Float::cast(node)?),
            SyntaxKind::INTEGER_BIN => Self::IntegerBin(IntegerBin::cast(node)?),
            SyntaxKind::INTEGER_DEC => Self::IntegerDec(IntegerDec::cast(node)?),
            SyntaxKind::INTEGER_HEX => Self::IntegerHex(IntegerHex::cast(node)?),
            SyntaxKind::INTEGER_OCT => Self::IntegerOct(IntegerOct::cast(node)?),
            SyntaxKind::LITERAL_STRING => Self::LiteralString(LiteralString::cast(node)?),
            SyntaxKind::LOCAL_DATE => Self::LocalDate(LocalDate::cast(node)?),
            SyntaxKind::LOCAL_DATE_TIME => Self::LocalDateTime(LocalDateTime::cast(node)?),
            SyntaxKind::LOCAL_TIME => Self::LocalTime(LocalTime::cast(node)?),
            SyntaxKind::MULTI_LINE_BASIC_STRING => {
                Self::MultiLineBasicString(MultiLineBasicString::cast(node)?)
            }
            SyntaxKind::MULTI_LINE_LITERAL_STRING => {
                Self::MultiLineLiteralString(MultiLineLiteralString::cast(node)?)
            }
            SyntaxKind::OFFSET_DATE_TIME => Self::OffsetDateTime(OffsetDateTime::cast(node)?),
            SyntaxKind::INVALID_TOKEN | SyntaxKind::ERROR => Self::Invalid(node.span()),
            _ => return None,
        })
    }

    pub fn span(&self) -> tombi_text::Span {
        match self {
            Self::Root(node) => node.span(),
            Self::Table(node) => node.span(),
            Self::ArrayOfTable(node) => node.span(),
            Self::KeyValue(node) => node.span(),
            Self::Keys(node) => node.span(),
            Self::Array(node) => node.span(),
            Self::InlineTable(node) => node.span(),
            Self::BasicString(node) => node.span(),
            Self::Boolean(node) => node.span(),
            Self::Float(node) => node.span(),
            Self::IntegerBin(node) => node.span(),
            Self::IntegerDec(node) => node.span(),
            Self::IntegerHex(node) => node.span(),
            Self::IntegerOct(node) => node.span(),
            Self::LiteralString(node) => node.span(),
            Self::LocalDate(node) => node.span(),
            Self::LocalDateTime(node) => node.span(),
            Self::LocalTime(node) => node.span(),
            Self::MultiLineBasicString(node) => node.span(),
            Self::MultiLineLiteralString(node) => node.span(),
            Self::OffsetDateTime(node) => node.span(),
            Self::Invalid(span) => *span,
        }
    }
}
