use tombi_json_lexer::Error as LexerError;
use tombi_json_syntax::SyntaxKind;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lexer error")]
    Lexer,
    #[error("Unexpected token: expected {expected:?}, got {actual:?}")]
    UnexpectedToken {
        expected: SyntaxKind,
        actual: SyntaxKind,
    },
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Invalid value")]
    InvalidValue,
    #[error("Maximum nesting depth ({limit}) exceeded")]
    RecursionLimitExceeded { limit: usize },
    #[error("Expected colon")]
    ExpectedColon,
    #[error("Duplicate key: {0}")]
    DuplicateKey(String),
    #[error("Invalid escape sequence")]
    InvalidEscapeSequence,
    #[error("Invalid Unicode escape sequence")]
    InvalidUnicodeEscape,
    #[error("Invalid Unicode code point")]
    InvalidUnicodeCodePoint,
}

impl From<LexerError> for Error {
    fn from(_: LexerError) -> Self {
        Error::Lexer
    }
}
