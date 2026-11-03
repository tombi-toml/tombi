#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ErrorKind {
    // tombi_lexer::ErrorKind translation
    #[error("invalid key")]
    InvalidKey,

    #[error("invalid basic string")]
    InvalidBasicString,

    #[error("invalid literal string")]
    InvalidLiteralString,

    #[error("invalid multi-line basic strings")]
    InvalidMultilineBasicString,

    #[error("invalid multi-line literal strings")]
    InvalidMultilineLiteralString,

    #[error("invalid number")]
    InvalidNumber,

    #[error("invalid offset date-time")]
    InvalidOffsetDateTime,

    #[error("invalid local date-time")]
    InvalidLocalDateTime,

    #[error("invalid local date")]
    InvalidLocalDate,

    #[error("invalid local time")]
    InvalidLocalTime,

    #[error("invalid line break")]
    InvalidLineBreak,

    #[error("invalid token")]
    InvalidToken,

    // Grammar error
    #[error("unknown line")]
    UnknownLine,

    #[error("expected key")]
    ExpectedKey,

    #[error("expected value")]
    ExpectedValue,

    #[error("expected '='")]
    ExpectedEqual,

    #[error("expected ']'")]
    ExpectedBracketEnd,

    #[error("expected ']]'")]
    ExpectedDoubleBracketEnd,

    #[error("expected '}}'")]
    ExpectedBraceEnd,

    #[error("expected line break")]
    ExpectedLineBreak,

    #[error("forbidden last period in keys")]
    ForbiddenKeyTrailingPeriod,

    #[error("maximum nesting depth exceeded")]
    RecursionLimitExceeded,
}

#[derive(thiserror::Error, Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    span: tombi_text::Span,
}

impl Error {
    pub fn new(kind: ErrorKind, span: tombi_text::Span) -> Self {
        Self { kind, span }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    #[inline]
    pub fn code(&self) -> &'static str {
        match self.kind {
            ErrorKind::InvalidKey => "syntax/invalid-key",
            ErrorKind::InvalidBasicString => "syntax/invalid-basic-string",
            ErrorKind::InvalidLiteralString => "syntax/invalid-literal-string",
            ErrorKind::InvalidMultilineBasicString => "syntax/invalid-multiline-basic-string",
            ErrorKind::InvalidMultilineLiteralString => "syntax/invalid-multiline-literal-string",
            ErrorKind::InvalidNumber => "syntax/invalid-number",
            ErrorKind::InvalidOffsetDateTime => "syntax/invalid-offset-date-time",
            ErrorKind::InvalidLocalDateTime => "syntax/invalid-local-date-time",
            ErrorKind::InvalidLocalDate => "syntax/invalid-local-date",
            ErrorKind::InvalidLocalTime => "syntax/invalid-local-time",
            ErrorKind::InvalidLineBreak => "syntax/invalid-line-break",
            ErrorKind::InvalidToken => "syntax/invalid-token",
            ErrorKind::UnknownLine => "syntax/unknown-line",
            ErrorKind::ExpectedKey => "syntax/expected-key",
            ErrorKind::ExpectedValue => "syntax/expected-value",
            ErrorKind::ExpectedEqual => "syntax/expected-equal",
            ErrorKind::ExpectedBracketEnd => "syntax/expected-bracket-end",
            ErrorKind::ExpectedDoubleBracketEnd => "syntax/expected-double-bracket-end",
            ErrorKind::ExpectedBraceEnd => "syntax/expected-brace-end",
            ErrorKind::ExpectedLineBreak => "syntax/expected-line-break",
            ErrorKind::ForbiddenKeyTrailingPeriod => "syntax/forbidden-key-trailing-period",
            ErrorKind::RecursionLimitExceeded => "syntax/recursion-limit-exceeded",
        }
    }

    pub fn to_message(&self) -> String {
        self.kind.to_string()
    }

    pub fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in {}", self.kind, self.span)
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.span == other.span
    }
}

impl Eq for Error {}

impl From<tombi_lexer::Error> for Error {
    fn from(error: tombi_lexer::Error) -> Self {
        let kind = match error.kind() {
            tombi_lexer::ErrorKind::InvalidKey => ErrorKind::InvalidKey,
            tombi_lexer::ErrorKind::InvalidBasicString => ErrorKind::InvalidBasicString,
            tombi_lexer::ErrorKind::InvalidLiteralString => ErrorKind::InvalidLiteralString,
            tombi_lexer::ErrorKind::InvalidMultilineBasicString => {
                ErrorKind::InvalidMultilineBasicString
            }
            tombi_lexer::ErrorKind::InvalidMultilineLiteralString => {
                ErrorKind::InvalidMultilineLiteralString
            }
            tombi_lexer::ErrorKind::InvalidNumber => ErrorKind::InvalidNumber,
            tombi_lexer::ErrorKind::InvalidOffsetDateTime => ErrorKind::InvalidOffsetDateTime,
            tombi_lexer::ErrorKind::InvalidLocalDateTime => ErrorKind::InvalidLocalDateTime,
            tombi_lexer::ErrorKind::InvalidLocalDate => ErrorKind::InvalidLocalDate,
            tombi_lexer::ErrorKind::InvalidLocalTime => ErrorKind::InvalidLocalTime,
            tombi_lexer::ErrorKind::InvalidLineBreak => ErrorKind::InvalidLineBreak,
            tombi_lexer::ErrorKind::InvalidToken => ErrorKind::InvalidToken,
        };

        Self::new(kind, error.span())
    }
}

#[cfg(feature = "diagnostic")]
impl tombi_diagnostic::SetDiagnostics for Error {
    #[inline]
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
            self.to_message(),
            self.code(),
            self.span(),
        ));
    }
}
