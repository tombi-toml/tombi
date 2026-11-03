#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    #[error("duplicate key: {key}")]
    DuplicateKey { key: String, span: tombi_text::Span },

    #[error("conflicting table")]
    TableConflict {
        range1: tombi_text::Span,
        range2: tombi_text::Span,
    },

    #[error("conflicting array")]
    ArrayConflict {
        range1: tombi_text::Span,
        range2: tombi_text::Span,
    },

    #[error("invalid integer: {error}")]
    InvalidInteger {
        error: std::num::ParseIntError,
        span: tombi_text::Span,
    },

    #[error("invalid float: {error}")]
    InvalidFloat {
        error: crate::support::float::ParseError,
        span: tombi_text::Span,
    },

    #[error("invalid string: {error}")]
    InvalidString {
        error: tombi_toml_text::ParseError,
        span: tombi_text::Span,
    },

    #[error("invalid offset date time: {error}")]
    InvalidOffsetDateTime {
        error: crate::support::chrono::ParseError,
        span: tombi_text::Span,
    },

    #[error("invalid local date time: {error}")]
    InvalidLocalDateTime {
        error: crate::support::chrono::ParseError,
        span: tombi_text::Span,
    },

    #[error("invalid local date: {error}")]
    InvalidLocalDate {
        error: crate::support::chrono::ParseError,
        span: tombi_text::Span,
    },

    #[error("invalid local time: {error}")]
    InvalidLocalTime {
        error: crate::support::chrono::ParseError,
        span: tombi_text::Span,
    },

    #[error("invalid date-time: {error}")]
    InvalidDateTime {
        error: tombi_date_time::parse::Error,
        span: tombi_text::Span,
    },

    #[error("invalid comment: {error}")]
    InvalidCommentSyntax {
        error: crate::support::comment::ParseError,
        span: tombi_text::Span,
    },

    /// Error when `ast::Node` is None
    #[error("incomplete node")]
    IncompleteNode { span: tombi_text::Span },
}

impl Error {
    pub fn to_message(&self) -> String {
        self.to_string()
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::DuplicateKey { .. } => "toml/duplicate-key",
            Self::TableConflict { .. } => "toml/table-conflict",
            Self::ArrayConflict { .. } => "toml/array-conflict",
            Self::InvalidInteger { .. } => "toml/invalid-integer",
            Self::InvalidFloat { .. } => "toml/invalid-float",
            Self::InvalidString { .. } => "toml/invalid-string",
            Self::InvalidOffsetDateTime { .. } => "toml/invalid-offset-date-time",
            Self::InvalidLocalDateTime { .. } => "toml/invalid-local-date-time",
            Self::InvalidLocalDate { .. } => "toml/invalid-local-date",
            Self::InvalidLocalTime { .. } => "toml/invalid-local-time",
            Self::InvalidDateTime { .. } => "toml/invalid-date-time",
            Self::InvalidCommentSyntax { .. } => "comment/invalid-syntax",
            Self::IncompleteNode { .. } => "toml/incomplete-node",
        }
    }

    pub fn span(&self) -> tombi_text::Span {
        match self {
            Self::DuplicateKey { span, .. } => *span,
            Self::TableConflict { range2, .. } => *range2,
            Self::ArrayConflict { range2, .. } => *range2,
            Self::InvalidInteger { span, .. } => *span,
            Self::InvalidFloat { span, .. } => *span,
            Self::InvalidString { span, .. } => *span,
            Self::InvalidOffsetDateTime { span, .. } => *span,
            Self::InvalidLocalDateTime { span, .. } => *span,
            Self::InvalidLocalDate { span, .. } => *span,
            Self::InvalidLocalTime { span, .. } => *span,
            Self::InvalidCommentSyntax { span, .. } => *span,
            Self::IncompleteNode { span } => *span,
            Self::InvalidDateTime { span, .. } => *span,
        }
    }
}

#[cfg(feature = "diagnostic")]
impl tombi_diagnostic::SetDiagnostics for Error {
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        match self {
            Self::ArrayConflict { range1, range2 } => {
                let diagnostic1 =
                    tombi_diagnostic::Diagnostic::new_error(self.to_message(), self.code(), range1);
                if !diagnostics.contains(&diagnostic1) {
                    diagnostics.push(diagnostic1);
                }
                diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
                    self.to_message(),
                    self.code(),
                    range2,
                ));
            }
            _ => {
                diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
                    self.to_message(),
                    self.code(),
                    self.span(),
                ));
            }
        }
    }
}
