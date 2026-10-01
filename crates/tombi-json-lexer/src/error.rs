#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    kind: ErrorKind,
    span: tombi_text::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidString,
    InvalidNumber,
    InvalidTrue,
    InvalidFalse,
    InvalidNull,
    InvalidToken,
    UnexpectedEndOfString,
    UnexpectedEscapeSequence,
    InvalidUnicodeEscapeSequence,
    InvalidLineBreak,
}

impl Error {
    #[inline]
    pub fn new(kind: ErrorKind, span: tombi_text::Span) -> Self {
        Self { kind, span }
    }

    #[inline]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.span
    }
}
