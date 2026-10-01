//! Generated file, do not edit by hand, see `xtask/src/codegen`

use crate::AstNode;
use crate::support;
use tombi_ast_syntax::{SyntaxKind, SyntaxKind::*, SyntaxNode, SyntaxToken, T};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Array<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> Array<'t> {
    #[inline]
    pub fn bracket_start(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T!['['])
    }
    #[inline]
    pub fn bracket_end(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T![']'])
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayOfTable<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> ArrayOfTable<'t> {
    #[inline]
    pub fn header(&self) -> Option<Keys<'t>> {
        support::node::child(&self.syntax)
    }
    #[inline]
    pub fn double_bracket_start(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T!["[["])
    }
    #[inline]
    pub fn double_bracket_end(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T!["]]"])
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BareKey<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> BareKey<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, BARE_KEY)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicString<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> BasicString<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, BASIC_STRING)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Boolean<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> Boolean<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, BOOLEAN)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Comma<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> Comma<'t> {
    #[inline]
    pub fn comma(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T ! [,])
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Float<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> Float<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, FLOAT)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InlineTable<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> InlineTable<'t> {
    #[inline]
    pub fn brace_start(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T!['{'])
    }
    #[inline]
    pub fn brace_end(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T!['}'])
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntegerBin<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> IntegerBin<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, INTEGER_BIN)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntegerDec<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> IntegerDec<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, INTEGER_DEC)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntegerHex<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> IntegerHex<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, INTEGER_HEX)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntegerOct<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> IntegerOct<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, INTEGER_OCT)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyValue<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> KeyValue<'t> {
    #[inline]
    pub fn keys(&self) -> Option<Keys<'t>> {
        support::node::child(&self.syntax)
    }
    #[inline]
    pub fn value(&self) -> Option<Value<'t>> {
        support::node::child(&self.syntax)
    }
    #[inline]
    pub fn eq(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T ! [=])
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keys<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> Keys<'t> {
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = Key<'t>> + use<'t> {
        self.syntax.child_nodes().filter_map(Key::cast)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LiteralString<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> LiteralString<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, LITERAL_STRING)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalDate<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> LocalDate<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, LOCAL_DATE)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalDateTime<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> LocalDateTime<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, LOCAL_DATE_TIME)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalTime<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> LocalTime<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, LOCAL_TIME)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MultiLineBasicString<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> MultiLineBasicString<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, MULTI_LINE_BASIC_STRING)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MultiLineLiteralString<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> MultiLineLiteralString<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, MULTI_LINE_LITERAL_STRING)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OffsetDateTime<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> OffsetDateTime<'t> {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, OFFSET_DATE_TIME)
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Root<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> Root<'t> {
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Table<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}
impl<'t> Table<'t> {
    #[inline]
    pub fn header(&self) -> Option<Keys<'t>> {
        support::node::child(&self.syntax)
    }
    #[inline]
    pub fn bracket_start(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T!['['])
    }
    #[inline]
    pub fn bracket_end(&self) -> Option<SyntaxToken<'t>> {
        support::node::token(&self.syntax, T![']'])
    }
    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key<'t> {
    BareKey(BareKey<'t>),
    BasicString(BasicString<'t>),
    LiteralString(LiteralString<'t>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RootItem<'t> {
    ArrayOfTable(ArrayOfTable<'t>),
    KeyValue(KeyValue<'t>),
    Table(Table<'t>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Value<'t> {
    Array(Array<'t>),
    BasicString(BasicString<'t>),
    Boolean(Boolean<'t>),
    Float(Float<'t>),
    InlineTable(InlineTable<'t>),
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
}
impl<'t> AstNode<'t> for Array<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ARRAY
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for ArrayOfTable<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ARRAY_OF_TABLE
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for BareKey<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BARE_KEY
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for BasicString<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BASIC_STRING
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for Boolean<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BOOLEAN
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for Comma<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::COMMA
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for Float<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOAT
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for InlineTable<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INLINE_TABLE
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for IntegerBin<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INTEGER_BIN
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for IntegerDec<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INTEGER_DEC
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for IntegerHex<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INTEGER_HEX
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for IntegerOct<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INTEGER_OCT
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for KeyValue<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::KEY_VALUE
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for Keys<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::KEYS
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for LiteralString<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LITERAL_STRING
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for LocalDate<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LOCAL_DATE
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for LocalDateTime<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LOCAL_DATE_TIME
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for LocalTime<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LOCAL_TIME
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for MultiLineBasicString<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::MULTI_LINE_BASIC_STRING
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for MultiLineLiteralString<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::MULTI_LINE_LITERAL_STRING
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for OffsetDateTime<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::OFFSET_DATE_TIME
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for Root<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ROOT
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> AstNode<'t> for Table<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::TABLE
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
impl<'t> From<BareKey<'t>> for Key<'t> {
    #[inline]
    fn from(node: BareKey<'t>) -> Key<'t> {
        Key::BareKey(node)
    }
}
impl<'t> From<BasicString<'t>> for Key<'t> {
    #[inline]
    fn from(node: BasicString<'t>) -> Key<'t> {
        Key::BasicString(node)
    }
}
impl<'t> From<LiteralString<'t>> for Key<'t> {
    #[inline]
    fn from(node: LiteralString<'t>) -> Key<'t> {
        Key::LiteralString(node)
    }
}
impl<'t> AstNode<'t> for Key<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::BARE_KEY | SyntaxKind::BASIC_STRING | SyntaxKind::LITERAL_STRING
        )
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        let res = match syntax.kind() {
            SyntaxKind::BARE_KEY => Key::BareKey(BareKey { syntax }),
            SyntaxKind::BASIC_STRING => Key::BasicString(BasicString { syntax }),
            SyntaxKind::LITERAL_STRING => Key::LiteralString(LiteralString { syntax }),
            _ => return None,
        };
        Some(res)
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        match self {
            Key::BareKey(it) => &it.syntax,
            Key::BasicString(it) => &it.syntax,
            Key::LiteralString(it) => &it.syntax,
        }
    }
}
impl<'t> From<ArrayOfTable<'t>> for RootItem<'t> {
    #[inline]
    fn from(node: ArrayOfTable<'t>) -> RootItem<'t> {
        RootItem::ArrayOfTable(node)
    }
}
impl<'t> From<KeyValue<'t>> for RootItem<'t> {
    #[inline]
    fn from(node: KeyValue<'t>) -> RootItem<'t> {
        RootItem::KeyValue(node)
    }
}
impl<'t> From<Table<'t>> for RootItem<'t> {
    #[inline]
    fn from(node: Table<'t>) -> RootItem<'t> {
        RootItem::Table(node)
    }
}
impl<'t> AstNode<'t> for RootItem<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::ARRAY_OF_TABLE | SyntaxKind::KEY_VALUE | SyntaxKind::TABLE
        )
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        let res = match syntax.kind() {
            SyntaxKind::ARRAY_OF_TABLE => RootItem::ArrayOfTable(ArrayOfTable { syntax }),
            SyntaxKind::KEY_VALUE => RootItem::KeyValue(KeyValue { syntax }),
            SyntaxKind::TABLE => RootItem::Table(Table { syntax }),
            _ => return None,
        };
        Some(res)
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        match self {
            RootItem::ArrayOfTable(it) => &it.syntax,
            RootItem::KeyValue(it) => &it.syntax,
            RootItem::Table(it) => &it.syntax,
        }
    }
}
impl<'t> From<Array<'t>> for Value<'t> {
    #[inline]
    fn from(node: Array<'t>) -> Value<'t> {
        Value::Array(node)
    }
}
impl<'t> From<BasicString<'t>> for Value<'t> {
    #[inline]
    fn from(node: BasicString<'t>) -> Value<'t> {
        Value::BasicString(node)
    }
}
impl<'t> From<Boolean<'t>> for Value<'t> {
    #[inline]
    fn from(node: Boolean<'t>) -> Value<'t> {
        Value::Boolean(node)
    }
}
impl<'t> From<Float<'t>> for Value<'t> {
    #[inline]
    fn from(node: Float<'t>) -> Value<'t> {
        Value::Float(node)
    }
}
impl<'t> From<InlineTable<'t>> for Value<'t> {
    #[inline]
    fn from(node: InlineTable<'t>) -> Value<'t> {
        Value::InlineTable(node)
    }
}
impl<'t> From<IntegerBin<'t>> for Value<'t> {
    #[inline]
    fn from(node: IntegerBin<'t>) -> Value<'t> {
        Value::IntegerBin(node)
    }
}
impl<'t> From<IntegerDec<'t>> for Value<'t> {
    #[inline]
    fn from(node: IntegerDec<'t>) -> Value<'t> {
        Value::IntegerDec(node)
    }
}
impl<'t> From<IntegerHex<'t>> for Value<'t> {
    #[inline]
    fn from(node: IntegerHex<'t>) -> Value<'t> {
        Value::IntegerHex(node)
    }
}
impl<'t> From<IntegerOct<'t>> for Value<'t> {
    #[inline]
    fn from(node: IntegerOct<'t>) -> Value<'t> {
        Value::IntegerOct(node)
    }
}
impl<'t> From<LiteralString<'t>> for Value<'t> {
    #[inline]
    fn from(node: LiteralString<'t>) -> Value<'t> {
        Value::LiteralString(node)
    }
}
impl<'t> From<LocalDate<'t>> for Value<'t> {
    #[inline]
    fn from(node: LocalDate<'t>) -> Value<'t> {
        Value::LocalDate(node)
    }
}
impl<'t> From<LocalDateTime<'t>> for Value<'t> {
    #[inline]
    fn from(node: LocalDateTime<'t>) -> Value<'t> {
        Value::LocalDateTime(node)
    }
}
impl<'t> From<LocalTime<'t>> for Value<'t> {
    #[inline]
    fn from(node: LocalTime<'t>) -> Value<'t> {
        Value::LocalTime(node)
    }
}
impl<'t> From<MultiLineBasicString<'t>> for Value<'t> {
    #[inline]
    fn from(node: MultiLineBasicString<'t>) -> Value<'t> {
        Value::MultiLineBasicString(node)
    }
}
impl<'t> From<MultiLineLiteralString<'t>> for Value<'t> {
    #[inline]
    fn from(node: MultiLineLiteralString<'t>) -> Value<'t> {
        Value::MultiLineLiteralString(node)
    }
}
impl<'t> From<OffsetDateTime<'t>> for Value<'t> {
    #[inline]
    fn from(node: OffsetDateTime<'t>) -> Value<'t> {
        Value::OffsetDateTime(node)
    }
}
impl<'t> AstNode<'t> for Value<'t> {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::ARRAY
                | SyntaxKind::BASIC_STRING
                | SyntaxKind::BOOLEAN
                | SyntaxKind::FLOAT
                | SyntaxKind::INLINE_TABLE
                | SyntaxKind::INTEGER_BIN
                | SyntaxKind::INTEGER_DEC
                | SyntaxKind::INTEGER_HEX
                | SyntaxKind::INTEGER_OCT
                | SyntaxKind::LITERAL_STRING
                | SyntaxKind::LOCAL_DATE
                | SyntaxKind::LOCAL_DATE_TIME
                | SyntaxKind::LOCAL_TIME
                | SyntaxKind::MULTI_LINE_BASIC_STRING
                | SyntaxKind::MULTI_LINE_LITERAL_STRING
                | SyntaxKind::OFFSET_DATE_TIME
        )
    }
    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        let res = match syntax.kind() {
            SyntaxKind::ARRAY => Value::Array(Array { syntax }),
            SyntaxKind::BASIC_STRING => Value::BasicString(BasicString { syntax }),
            SyntaxKind::BOOLEAN => Value::Boolean(Boolean { syntax }),
            SyntaxKind::FLOAT => Value::Float(Float { syntax }),
            SyntaxKind::INLINE_TABLE => Value::InlineTable(InlineTable { syntax }),
            SyntaxKind::INTEGER_BIN => Value::IntegerBin(IntegerBin { syntax }),
            SyntaxKind::INTEGER_DEC => Value::IntegerDec(IntegerDec { syntax }),
            SyntaxKind::INTEGER_HEX => Value::IntegerHex(IntegerHex { syntax }),
            SyntaxKind::INTEGER_OCT => Value::IntegerOct(IntegerOct { syntax }),
            SyntaxKind::LITERAL_STRING => Value::LiteralString(LiteralString { syntax }),
            SyntaxKind::LOCAL_DATE => Value::LocalDate(LocalDate { syntax }),
            SyntaxKind::LOCAL_DATE_TIME => Value::LocalDateTime(LocalDateTime { syntax }),
            SyntaxKind::LOCAL_TIME => Value::LocalTime(LocalTime { syntax }),
            SyntaxKind::MULTI_LINE_BASIC_STRING => {
                Value::MultiLineBasicString(MultiLineBasicString { syntax })
            }
            SyntaxKind::MULTI_LINE_LITERAL_STRING => {
                Value::MultiLineLiteralString(MultiLineLiteralString { syntax })
            }
            SyntaxKind::OFFSET_DATE_TIME => Value::OffsetDateTime(OffsetDateTime { syntax }),
            _ => return None,
        };
        Some(res)
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        match self {
            Value::Array(it) => &it.syntax,
            Value::BasicString(it) => &it.syntax,
            Value::Boolean(it) => &it.syntax,
            Value::Float(it) => &it.syntax,
            Value::InlineTable(it) => &it.syntax,
            Value::IntegerBin(it) => &it.syntax,
            Value::IntegerDec(it) => &it.syntax,
            Value::IntegerHex(it) => &it.syntax,
            Value::IntegerOct(it) => &it.syntax,
            Value::LiteralString(it) => &it.syntax,
            Value::LocalDate(it) => &it.syntax,
            Value::LocalDateTime(it) => &it.syntax,
            Value::LocalTime(it) => &it.syntax,
            Value::MultiLineBasicString(it) => &it.syntax,
            Value::MultiLineLiteralString(it) => &it.syntax,
            Value::OffsetDateTime(it) => &it.syntax,
        }
    }
}
impl std::fmt::Display for Key<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for RootItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Array<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ArrayOfTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BareKey<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BasicString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Boolean<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Comma<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Float<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for InlineTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IntegerBin<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IntegerDec<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IntegerHex<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IntegerOct<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for KeyValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Keys<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LiteralString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LocalDate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LocalDateTime<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LocalTime<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for MultiLineBasicString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for MultiLineLiteralString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for OffsetDateTime<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Root<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Table<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
