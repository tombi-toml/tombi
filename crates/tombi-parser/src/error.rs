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

    #[error("expected ','")]
    ExpectedComma,

    #[error("expected ']'")]
    ExpectedBracketEnd,

    #[error("expected ']]'")]
    ExpectedDoubleBracketEnd,

    #[error("expected '}}'")]
    ExpectedBraceEnd,

    #[error("expected line break")]
    ExpectedLineBreak,

    #[error("forbidden last period in keys")]
    ForbiddenKeysLastPeriod,

    #[error("inline table must be single line in TOML v1.0.0 or earlier")]
    InlineTableMustSingleLine,

    #[error("trailing comma in inline table not allowed in TOML v1.0.0 or earlier")]
    ForbiddenInlineTableLastComma,
}

#[derive(thiserror::Error, Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    range: tombi_text::Range,
}

impl Error {
    pub fn new(kind: ErrorKind, range: tombi_text::Range) -> Self {
        Self { kind, range }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    #[inline]
    pub fn code(&self) -> &'static str {
        match self.kind {
            ErrorKind::InvalidKey => "invalid-key",
            ErrorKind::InvalidBasicString => "invalid-basic-string",
            ErrorKind::InvalidLiteralString => "invalid-literal-string",
            ErrorKind::InvalidMultilineBasicString => "invalid-multiline-basic-string",
            ErrorKind::InvalidMultilineLiteralString => "invalid-multiline-literal-string",
            ErrorKind::InvalidNumber => "invalid-number",
            ErrorKind::InvalidOffsetDateTime => "invalid-offset-date-time",
            ErrorKind::InvalidLocalDateTime => "invalid-local-date-time",
            ErrorKind::InvalidLocalDate => "invalid-local-date",
            ErrorKind::InvalidLocalTime => "invalid-local-time",
            ErrorKind::InvalidLineBreak => "invalid-line-break",
            ErrorKind::InvalidToken => "invalid-token",
            ErrorKind::UnknownLine => "unknown-line",
            ErrorKind::ExpectedKey => "expected-key",
            ErrorKind::ExpectedValue => "expected-value",
            ErrorKind::ExpectedEqual => "expected-equal",
            ErrorKind::ExpectedComma => "expected-comma",
            ErrorKind::ExpectedBracketEnd => "expected-bracket-end",
            ErrorKind::ExpectedDoubleBracketEnd => "expected-double-bracket-end",
            ErrorKind::ExpectedBraceEnd => "expected-brace-end",
            ErrorKind::ExpectedLineBreak => "expected-line-break",
            ErrorKind::ForbiddenKeysLastPeriod => "forbidden-keys-last-period",
            ErrorKind::InlineTableMustSingleLine => "inline-table-must-single-line",
            ErrorKind::ForbiddenInlineTableLastComma => "forbidden-inline-table-last-comma",
        }
    }

    pub fn to_message(&self) -> String {
        self.kind.to_string()
    }

    pub fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} in {}..{}",
            self.kind, self.range.start, self.range.end
        )
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.range == other.range
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

        Self::new(kind, error.range())
    }
}

#[cfg(feature = "diagnostic")]
impl tombi_diagnostic::SetDiagnostics for Error {
    /// Converts an error into diagnostic information.
    ///
    /// Since a single error may be converted into multiple diagnostics,
    /// this function takes a vector of diagnostics and appends the conversion results to it.
    #[inline]
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
            self.to_message(),
            self.code(),
            self.range(),
        ));
    }
}
