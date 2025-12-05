mod language;

pub use language::JsonLanguage;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,
    // JSON basic tokens
    COMMA,
    COLON,
    BRACKET_START,
    BRACKET_END,
    BRACE_START,
    BRACE_END,
    // JSON values
    NUMBER,
    STRING,
    BOOLEAN,
    NULL,
    // Trivia
    WHITESPACE,
    LINE_BREAK,
    // Nodes
    ROOT,
    ARRAY,
    OBJECT,
    MEMBER,
    VALUE,
    #[doc(hidden)]
    INVALID_TOKEN,
    #[doc(hidden)]
    __LAST,
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::LINE_BREAK)
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

/// Utility macro for creating a SyntaxKind through simple macro syntax
#[macro_export]
macro_rules! T {
    ['{'] => { $crate::SyntaxKind::BRACE_START };
    ['}'] => { $crate::SyntaxKind::BRACE_END };
    ['['] => { $crate::SyntaxKind::BRACKET_START };
    [']'] => { $crate::SyntaxKind::BRACKET_END };
    [,] => { $crate::SyntaxKind::COMMA };
    [:] => { $crate::SyntaxKind::COLON };
}

pub type SyntaxNode = tombi_rg_tree::RedNode<JsonLanguage>;
pub type SyntaxToken = tombi_rg_tree::RedToken<JsonLanguage>;
pub type SyntaxElement = tombi_rg_tree::RedElement<JsonLanguage>;
pub type SyntaxNodePtr = tombi_rg_tree::RedNodePtr<JsonLanguage>;
pub type SyntaxNodeChildren = tombi_rg_tree::RedNodeChildren<JsonLanguage>;
pub type SyntaxElementChildren = tombi_rg_tree::RedElementChildren<JsonLanguage>;
pub type PreorderWithTokens = tombi_rg_tree::RedPreorderWithTokens<JsonLanguage>;
