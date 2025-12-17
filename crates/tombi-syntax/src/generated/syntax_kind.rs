//! Generated file, do not edit by hand, see `xtask/src/codegen`

#[doc = r" The kind of syntax node, e.g. `WHITESPACE`, `COMMENT`, or `TABLE`."]
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,
    COMMA,
    DOT,
    EQUAL,
    BRACKET_START,
    BRACKET_END,
    BRACE_START,
    BRACE_END,
    DOUBLE_BRACKET_START,
    DOUBLE_BRACKET_END,
    BASIC_STRING,
    MULTI_LINE_BASIC_STRING,
    LITERAL_STRING,
    MULTI_LINE_LITERAL_STRING,
    INTEGER_DEC,
    INTEGER_HEX,
    INTEGER_OCT,
    INTEGER_BIN,
    FLOAT,
    BOOLEAN,
    OFFSET_DATE_TIME,
    LOCAL_DATE_TIME,
    LOCAL_DATE,
    LOCAL_TIME,
    WHITESPACE,
    LINE_BREAK,
    COMMENT,
    ERROR,
    ROOT,
    KEYS,
    KEY,
    VALUE,
    KEY_VALUE,
    BARE_KEY,
    ARRAY,
    TABLE,
    INLINE_TABLE,
    ARRAY_OF_TABLE,
    #[doc(hidden)]
    INVALID_TOKEN,
    #[doc(hidden)]
    __LAST,
}
impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE)
    }
}
impl From<SyntaxKind> for tombi_rg_tree::SyntaxKind {
    #[inline]
    fn from(k: SyntaxKind) -> Self {
        Self(k as u16)
    }
}
impl From<u16> for SyntaxKind {
    #[inline]
    fn from(d: u16) -> SyntaxKind {
        debug_assert!(d <= (SyntaxKind::__LAST as u16));
        unsafe { std::mem::transmute::<u16, SyntaxKind>(d) }
    }
}
impl From<SyntaxKind> for u16 {
    #[inline]
    fn from(k: SyntaxKind) -> u16 {
        k as u16
    }
}
#[doc = r" Utility macro for creating a SyntaxKind through simple macro syntax"]
#[macro_export]
macro_rules ! T { [,] => { $ crate :: SyntaxKind :: COMMA } ; [.] => { $ crate :: SyntaxKind :: DOT } ; [=] => { $ crate :: SyntaxKind :: EQUAL } ; ['['] => { $ crate :: SyntaxKind :: BRACKET_START } ; [']'] => { $ crate :: SyntaxKind :: BRACKET_END } ; ['{'] => { $ crate :: SyntaxKind :: BRACE_START } ; ['}'] => { $ crate :: SyntaxKind :: BRACE_END } ; ["[["] => { $ crate :: SyntaxKind :: DOUBLE_BRACKET_START } ; ["]]"] => { $ crate :: SyntaxKind :: DOUBLE_BRACKET_END } ; [bare_key] => { $ crate :: SyntaxKind :: BARE_KEY } ; [basic_string] => { $ crate :: SyntaxKind :: BASIC_STRING } ; [multi_line_basic_string] => { $ crate :: SyntaxKind :: MULTI_LINE_BASIC_STRING } ; [literal_string] => { $ crate :: SyntaxKind :: LITERAL_STRING } ; [multi_line_literal_string] => { $ crate :: SyntaxKind :: MULTI_LINE_LITERAL_STRING } ; [integer_dec] => { $ crate :: SyntaxKind :: INTEGER_DEC } ; [integer_hex] => { $ crate :: SyntaxKind :: INTEGER_HEX } ; [integer_oct] => { $ crate :: SyntaxKind :: INTEGER_OCT } ; [integer_bin] => { $ crate :: SyntaxKind :: INTEGER_BIN } ; [float] => { $ crate :: SyntaxKind :: FLOAT } ; [offset_date_time] => { $ crate :: SyntaxKind :: OFFSET_DATE_TIME } ; [local_date_time] => { $ crate :: SyntaxKind :: LOCAL_DATE_TIME } ; [local_date] => { $ crate :: SyntaxKind :: LOCAL_DATE } ; [local_time] => { $ crate :: SyntaxKind :: LOCAL_TIME } ; }
