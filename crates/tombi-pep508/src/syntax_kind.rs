#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,

    // Identifiers and values
    IDENTIFIER,     // Package name, extra name, etc.
    VERSION_STRING, // Version number like "1.2.3"
    STRING,         // Quoted string
    URL,            // URL string

    // Operators
    EQ_EQ,    // ==
    NOT_EQ,   // !=
    LTE,      // <=
    GTE,      // >=
    LT,       // <
    GT,       // >
    TILDE_EQ, // ~=
    EQ_EQ_EQ, // ===

    // Separators
    COMMA,         // ,
    SEMICOLON,     // ;
    AT,            // @
    BRACKET_START, // [
    BRACKET_END,   // ]
    PAREN_START,   // (
    PAREN_END,     // )

    // Keywords
    AND, // and
    OR,  // or
    IN,  // in
    NOT, // not

    // Environment markers
    PYTHON_VERSION,
    PYTHON_FULL_VERSION,
    OS_NAME,
    SYS_PLATFORM,
    PLATFORM_RELEASE,
    PLATFORM_SYSTEM,
    PLATFORM_VERSION,
    PLATFORM_MACHINE,
    PLATFORM_PYTHON_IMPLEMENTATION,
    IMPLEMENTATION_NAME,
    IMPLEMENTATION_VERSION,
    EXTRA,

    // Trivia
    WHITESPACE,
    COMMENT,

    // Error recovery
    INVALID_TOKEN,

    #[doc(hidden)]
    __LAST,
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
    }

    #[inline]
    pub fn is_version_operator(self) -> bool {
        matches!(
            self,
            SyntaxKind::EQ_EQ
                | SyntaxKind::NOT_EQ
                | SyntaxKind::LTE
                | SyntaxKind::GTE
                | SyntaxKind::LT
                | SyntaxKind::GT
                | SyntaxKind::TILDE_EQ
                | SyntaxKind::EQ_EQ_EQ
        )
    }

    #[inline]
    pub fn is_marker_variable(self) -> bool {
        matches!(
            self,
            SyntaxKind::PYTHON_VERSION
                | SyntaxKind::PYTHON_FULL_VERSION
                | SyntaxKind::OS_NAME
                | SyntaxKind::SYS_PLATFORM
                | SyntaxKind::PLATFORM_RELEASE
                | SyntaxKind::PLATFORM_SYSTEM
                | SyntaxKind::PLATFORM_VERSION
                | SyntaxKind::PLATFORM_MACHINE
                | SyntaxKind::PLATFORM_PYTHON_IMPLEMENTATION
                | SyntaxKind::IMPLEMENTATION_NAME
                | SyntaxKind::IMPLEMENTATION_VERSION
                | SyntaxKind::EXTRA
        )
    }
}

/// Utility macro for creating a SyntaxKind through simple macro syntax
#[macro_export]
macro_rules! T {
    ['['] => { $crate::SyntaxKind::BRACKET_START };
    [']'] => { $crate::SyntaxKind::BRACKET_END };
    ['('] => { $crate::SyntaxKind::PAREN_START };
    [')'] => { $crate::SyntaxKind::PAREN_END };
    [,] => { $crate::SyntaxKind::COMMA };
    [;] => { $crate::SyntaxKind::SEMICOLON };
    [@] => { $crate::SyntaxKind::AT };
    [==] => { $crate::SyntaxKind::EQ_EQ };
    [!=] => { $crate::SyntaxKind::NOT_EQ };
    [<=] => { $crate::SyntaxKind::LTE };
    [>=] => { $crate::SyntaxKind::GTE };
    [<] => { $crate::SyntaxKind::LT };
    [>] => { $crate::SyntaxKind::GT };
    [~=] => { $crate::SyntaxKind::TILDE_EQ };
    [===] => { $crate::SyntaxKind::EQ_EQ_EQ };
    [and] => { $crate::SyntaxKind::AND };
    [or] => { $crate::SyntaxKind::OR };
    [in] => { $crate::SyntaxKind::IN };
    [not] => { $crate::SyntaxKind::NOT };
}
