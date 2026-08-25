use crate::{
    Array, ArrayOfTable, AstNode, BasicString, Boolean, Float, InlineTable, IntegerBin, IntegerDec,
    IntegerHex, IntegerOct, KeyValue, Keys, LiteralString, LocalDate, LocalDateTime, LocalTime,
    MultiLineBasicString, MultiLineLiteralString, OffsetDateTime, Root, SyntaxKind, SyntaxNode,
    Table,
};

/// A closed, TOML-specific view of a node in source order.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TomlNode {
    Root(Root),
    Table(Table),
    ArrayOfTable(ArrayOfTable),
    KeyValue(KeyValue),
    Keys(Keys),
    Array(Array),
    InlineTable(InlineTable),
    BasicString(BasicString),
    Boolean(Boolean),
    Float(Float),
    IntegerBin(IntegerBin),
    IntegerDec(IntegerDec),
    IntegerHex(IntegerHex),
    IntegerOct(IntegerOct),
    LiteralString(LiteralString),
    LocalDate(LocalDate),
    LocalDateTime(LocalDateTime),
    LocalTime(LocalTime),
    MultiLineBasicString(MultiLineBasicString),
    MultiLineLiteralString(MultiLineLiteralString),
    OffsetDateTime(OffsetDateTime),
    Invalid(tombi_text::Range),
}

/// Commas adjacent to the syntax item at a cursor position.
///
/// This includes commas recovered inside invalid syntax while the user is
/// typing an incomplete array or inline table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdjacentCommas {
    pub before: Option<tombi_text::Range>,
    pub after: Option<tombi_text::Range>,
}

impl TomlNode {
    pub(crate) fn cast(node: SyntaxNode) -> Option<Self> {
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
            SyntaxKind::INVALID_TOKEN | SyntaxKind::ERROR => Self::Invalid(node.range()),
            _ => return None,
        })
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            Self::Root(node) => node.range(),
            Self::Table(node) => node.range(),
            Self::ArrayOfTable(node) => node.range(),
            Self::KeyValue(node) => node.range(),
            Self::Keys(node) => node.range(),
            Self::Array(node) => node.range(),
            Self::InlineTable(node) => node.range(),
            Self::BasicString(node) => node.range(),
            Self::Boolean(node) => node.range(),
            Self::Float(node) => node.range(),
            Self::IntegerBin(node) => node.range(),
            Self::IntegerDec(node) => node.range(),
            Self::IntegerHex(node) => node.range(),
            Self::IntegerOct(node) => node.range(),
            Self::LiteralString(node) => node.range(),
            Self::LocalDate(node) => node.range(),
            Self::LocalDateTime(node) => node.range(),
            Self::LocalTime(node) => node.range(),
            Self::MultiLineBasicString(node) => node.range(),
            Self::MultiLineLiteralString(node) => node.range(),
            Self::OffsetDateTime(node) => node.range(),
            Self::Invalid(range) => *range,
        }
    }
}
