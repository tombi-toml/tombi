//! Generated file, do not edit by hand, see `xtask/src/codegen`

use crate::support;
use crate::AstChildren;
use crate::AstNode;
use tombi_syntax::{SyntaxKind, SyntaxKind::*, SyntaxNode, SyntaxToken, T};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array {
    pub(crate) syntax: SyntaxNode,
}
impl Array {
    #[inline]
    pub fn values(&self) -> AstChildren<Value> {
        support::node::children(&self.syntax)
    }
    #[inline]
    pub fn bracket_start(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T!['['])
    }
    #[inline]
    pub fn bracket_end(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T![']'])
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayOfTable {
    pub(crate) syntax: SyntaxNode,
}
impl ArrayOfTable {
    #[inline]
    pub fn header(&self) -> Option<Keys> {
        support::node::child(&self.syntax)
    }
    #[inline]
    pub fn key_values(&self) -> AstChildren<KeyValue> {
        support::node::children(&self.syntax)
    }
    #[inline]
    pub fn double_bracket_start(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T!["[["])
    }
    #[inline]
    pub fn double_bracket_end(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T!["]]"])
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BareKey {
    pub(crate) syntax: SyntaxNode,
}
impl BareKey {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, BARE_KEY)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicString {
    pub(crate) syntax: SyntaxNode,
}
impl BasicString {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, BASIC_STRING)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Boolean {
    pub(crate) syntax: SyntaxNode,
}
impl Boolean {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, BOOLEAN)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comma {
    pub(crate) syntax: SyntaxNode,
}
impl Comma {
    #[inline]
    pub fn comma(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T ! [,])
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Float {
    pub(crate) syntax: SyntaxNode,
}
impl Float {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, FLOAT)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineTable {
    pub(crate) syntax: SyntaxNode,
}
impl InlineTable {
    #[inline]
    pub fn key_values(&self) -> AstChildren<KeyValue> {
        support::node::children(&self.syntax)
    }
    #[inline]
    pub fn brace_start(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T!['{'])
    }
    #[inline]
    pub fn brace_end(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T!['}'])
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerBin {
    pub(crate) syntax: SyntaxNode,
}
impl IntegerBin {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, INTEGER_BIN)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerDec {
    pub(crate) syntax: SyntaxNode,
}
impl IntegerDec {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, INTEGER_DEC)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerHex {
    pub(crate) syntax: SyntaxNode,
}
impl IntegerHex {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, INTEGER_HEX)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerOct {
    pub(crate) syntax: SyntaxNode,
}
impl IntegerOct {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, INTEGER_OCT)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyValue {
    pub(crate) syntax: SyntaxNode,
}
impl KeyValue {
    #[inline]
    pub fn keys(&self) -> Option<Keys> {
        support::node::child(&self.syntax)
    }
    #[inline]
    pub fn value(&self) -> Option<Value> {
        support::node::child(&self.syntax)
    }
    #[inline]
    pub fn eq(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T ! [=])
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keys {
    pub(crate) syntax: SyntaxNode,
}
impl Keys {
    #[inline]
    pub fn keys(&self) -> AstChildren<Key> {
        support::node::children(&self.syntax)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralString {
    pub(crate) syntax: SyntaxNode,
}
impl LiteralString {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, LITERAL_STRING)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalDate {
    pub(crate) syntax: SyntaxNode,
}
impl LocalDate {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, LOCAL_DATE)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalDateTime {
    pub(crate) syntax: SyntaxNode,
}
impl LocalDateTime {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, LOCAL_DATE_TIME)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalTime {
    pub(crate) syntax: SyntaxNode,
}
impl LocalTime {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, LOCAL_TIME)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiLineBasicString {
    pub(crate) syntax: SyntaxNode,
}
impl MultiLineBasicString {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, MULTI_LINE_BASIC_STRING)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiLineLiteralString {
    pub(crate) syntax: SyntaxNode,
}
impl MultiLineLiteralString {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, MULTI_LINE_LITERAL_STRING)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OffsetDateTime {
    pub(crate) syntax: SyntaxNode,
}
impl OffsetDateTime {
    #[inline]
    pub fn token(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, OFFSET_DATE_TIME)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Root {
    pub(crate) syntax: SyntaxNode,
}
impl Root {
    #[inline]
    pub fn items(&self) -> AstChildren<RootItem> {
        support::node::children(&self.syntax)
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Table {
    pub(crate) syntax: SyntaxNode,
}
impl Table {
    #[inline]
    pub fn header(&self) -> Option<Keys> {
        support::node::child(&self.syntax)
    }
    #[inline]
    pub fn key_values(&self) -> AstChildren<KeyValue> {
        support::node::children(&self.syntax)
    }
    #[inline]
    pub fn bracket_start(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T!['['])
    }
    #[inline]
    pub fn bracket_end(&self) -> Option<SyntaxToken> {
        support::node::token(&self.syntax, T![']'])
    }
    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    BareKey(BareKey),
    BasicString(BasicString),
    LiteralString(LiteralString),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RootItem {
    ArrayOfTable(ArrayOfTable),
    KeyValue(KeyValue),
    Table(Table),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Array(Array),
    BasicString(BasicString),
    Boolean(Boolean),
    Float(Float),
    InlineTable(InlineTable),
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
}
impl AstNode for Array {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ARRAY
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ArrayOfTable {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ARRAY_OF_TABLE
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for BareKey {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BARE_KEY
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for BasicString {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BASIC_STRING
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Boolean {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BOOLEAN
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Comma {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::COMMA
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Float {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOAT
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for InlineTable {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INLINE_TABLE
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for IntegerBin {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INTEGER_BIN
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for IntegerDec {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INTEGER_DEC
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for IntegerHex {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INTEGER_HEX
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for IntegerOct {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INTEGER_OCT
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for KeyValue {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::KEY_VALUE
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Keys {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::KEYS
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for LiteralString {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LITERAL_STRING
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for LocalDate {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LOCAL_DATE
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for LocalDateTime {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LOCAL_DATE_TIME
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for LocalTime {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LOCAL_TIME
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for MultiLineBasicString {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::MULTI_LINE_BASIC_STRING
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for MultiLineLiteralString {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::MULTI_LINE_LITERAL_STRING
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for OffsetDateTime {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::OFFSET_DATE_TIME
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Root {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ROOT
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Table {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::TABLE
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl From<BareKey> for Key {
    #[inline]
    fn from(node: BareKey) -> Key {
        Key::BareKey(node)
    }
}
impl From<BasicString> for Key {
    #[inline]
    fn from(node: BasicString) -> Key {
        Key::BasicString(node)
    }
}
impl From<LiteralString> for Key {
    #[inline]
    fn from(node: LiteralString) -> Key {
        Key::LiteralString(node)
    }
}
impl AstNode for Key {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::BARE_KEY | SyntaxKind::BASIC_STRING | SyntaxKind::LITERAL_STRING
        )
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            SyntaxKind::BARE_KEY => Key::BareKey(BareKey { syntax }),
            SyntaxKind::BASIC_STRING => Key::BasicString(BasicString { syntax }),
            SyntaxKind::LITERAL_STRING => Key::LiteralString(LiteralString { syntax }),
            _ => return None,
        };
        Some(res)
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Key::BareKey(it) => &it.syntax,
            Key::BasicString(it) => &it.syntax,
            Key::LiteralString(it) => &it.syntax,
        }
    }
}
impl From<ArrayOfTable> for RootItem {
    #[inline]
    fn from(node: ArrayOfTable) -> RootItem {
        RootItem::ArrayOfTable(node)
    }
}
impl From<KeyValue> for RootItem {
    #[inline]
    fn from(node: KeyValue) -> RootItem {
        RootItem::KeyValue(node)
    }
}
impl From<Table> for RootItem {
    #[inline]
    fn from(node: Table) -> RootItem {
        RootItem::Table(node)
    }
}
impl AstNode for RootItem {
    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::ARRAY_OF_TABLE | SyntaxKind::KEY_VALUE | SyntaxKind::TABLE
        )
    }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            SyntaxKind::ARRAY_OF_TABLE => RootItem::ArrayOfTable(ArrayOfTable { syntax }),
            SyntaxKind::KEY_VALUE => RootItem::KeyValue(KeyValue { syntax }),
            SyntaxKind::TABLE => RootItem::Table(Table { syntax }),
            _ => return None,
        };
        Some(res)
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        match self {
            RootItem::ArrayOfTable(it) => &it.syntax,
            RootItem::KeyValue(it) => &it.syntax,
            RootItem::Table(it) => &it.syntax,
        }
    }
}
impl From<Array> for Value {
    #[inline]
    fn from(node: Array) -> Value {
        Value::Array(node)
    }
}
impl From<BasicString> for Value {
    #[inline]
    fn from(node: BasicString) -> Value {
        Value::BasicString(node)
    }
}
impl From<Boolean> for Value {
    #[inline]
    fn from(node: Boolean) -> Value {
        Value::Boolean(node)
    }
}
impl From<Float> for Value {
    #[inline]
    fn from(node: Float) -> Value {
        Value::Float(node)
    }
}
impl From<InlineTable> for Value {
    #[inline]
    fn from(node: InlineTable) -> Value {
        Value::InlineTable(node)
    }
}
impl From<IntegerBin> for Value {
    #[inline]
    fn from(node: IntegerBin) -> Value {
        Value::IntegerBin(node)
    }
}
impl From<IntegerDec> for Value {
    #[inline]
    fn from(node: IntegerDec) -> Value {
        Value::IntegerDec(node)
    }
}
impl From<IntegerHex> for Value {
    #[inline]
    fn from(node: IntegerHex) -> Value {
        Value::IntegerHex(node)
    }
}
impl From<IntegerOct> for Value {
    #[inline]
    fn from(node: IntegerOct) -> Value {
        Value::IntegerOct(node)
    }
}
impl From<LiteralString> for Value {
    #[inline]
    fn from(node: LiteralString) -> Value {
        Value::LiteralString(node)
    }
}
impl From<LocalDate> for Value {
    #[inline]
    fn from(node: LocalDate) -> Value {
        Value::LocalDate(node)
    }
}
impl From<LocalDateTime> for Value {
    #[inline]
    fn from(node: LocalDateTime) -> Value {
        Value::LocalDateTime(node)
    }
}
impl From<LocalTime> for Value {
    #[inline]
    fn from(node: LocalTime) -> Value {
        Value::LocalTime(node)
    }
}
impl From<MultiLineBasicString> for Value {
    #[inline]
    fn from(node: MultiLineBasicString) -> Value {
        Value::MultiLineBasicString(node)
    }
}
impl From<MultiLineLiteralString> for Value {
    #[inline]
    fn from(node: MultiLineLiteralString) -> Value {
        Value::MultiLineLiteralString(node)
    }
}
impl From<OffsetDateTime> for Value {
    #[inline]
    fn from(node: OffsetDateTime) -> Value {
        Value::OffsetDateTime(node)
    }
}
impl AstNode for Value {
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
    fn cast(syntax: SyntaxNode) -> Option<Self> {
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
    fn syntax(&self) -> &SyntaxNode {
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
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for RootItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ArrayOfTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BareKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BasicString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Comma {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for InlineTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IntegerBin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IntegerDec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IntegerHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IntegerOct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for KeyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LiteralString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LocalDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LocalDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LocalTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for MultiLineBasicString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for MultiLineLiteralString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for OffsetDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
