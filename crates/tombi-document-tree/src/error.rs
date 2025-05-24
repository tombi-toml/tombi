#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    #[error("duplicate key: {key}")]
    DuplicateKey {
        key: String,
        range: tombi_text::Range,
    },

    #[error("conflicting table")]
    ConflictTable {
        range1: tombi_text::Range,
        range2: tombi_text::Range,
    },

    #[error("conflicting array")]
    ConflictArray {
        range1: tombi_text::Range,
        range2: tombi_text::Range,
    },

    #[error("invalid integer: {error}")]
    ParseIntError {
        error: std::num::ParseIntError,
        range: tombi_text::Range,
    },

    #[error("invalid float: {error}")]
    ParseFloatError {
        error: crate::support::float::ParseError,
        range: tombi_text::Range,
    },

    #[error("invalid string: {error}")]
    ParseStringError {
        error: tombi_toml_text::ParseError,
        range: tombi_text::Range,
    },

    #[error("invalid offset date time: {error}")]
    ParseOffsetDateTimeError {
        error: crate::support::chrono::ParseError,
        range: tombi_text::Range,
    },

    #[error("invalid local date time: {error}")]
    ParseLocalDateTimeError {
        error: crate::support::chrono::ParseError,
        range: tombi_text::Range,
    },

    #[error("invalid local date: {error}")]
    ParseLocalDateError {
        error: crate::support::chrono::ParseError,
        range: tombi_text::Range,
    },

    #[error("invalid local time: {error}")]
    ParseLocalTimeError {
        error: crate::support::chrono::ParseError,
        range: tombi_text::Range,
    },

    #[error("invalid date-time: {error}")]
    ParseDateTimeError {
        error: tombi_date_time::parse::Error,
        range: tombi_text::Range,
    },

    #[error("invalid comment: {error}")]
    ParseCommentError {
        error: crate::support::comment::ParseError,
        range: tombi_text::Range,
    },

    /// Error when `ast::Node` is None
    #[error("incomplete node")]
    IncompleteNode { range: tombi_text::Range },
}

impl Error {
    pub fn to_message(&self) -> String {
        self.to_string()
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            Self::DuplicateKey { range, .. } => *range,
            Self::ConflictTable { range2, .. } => *range2,
            Self::ConflictArray { range2, .. } => *range2,
            Self::ParseIntError { range, .. } => *range,
            Self::ParseFloatError { range, .. } => *range,
            Self::ParseStringError { range, .. } => *range,
            Self::ParseOffsetDateTimeError { range, .. } => *range,
            Self::ParseLocalDateTimeError { range, .. } => *range,
            Self::ParseLocalDateError { range, .. } => *range,
            Self::ParseLocalTimeError { range, .. } => *range,
            Self::ParseCommentError { range, .. } => *range,
            Self::IncompleteNode { range } => *range,
            Self::ParseDateTimeError { range, .. } => *range,
        }
    }
}

#[cfg(feature = "diagnostic")]
impl tombi_diagnostic::SetDiagnostics for Error {
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        match self {
            Self::ConflictArray { range1, range2 } => {
                let diagnostic1 =
                    tombi_diagnostic::Diagnostic::new_error(self.to_message(), range1);
                if !diagnostics.contains(&diagnostic1) {
                    diagnostics.push(diagnostic1);
                }
                diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
                    self.to_message(),
                    range2,
                ));
            }
            _ => {
                diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
                    self.to_message(),
                    self.range(),
                ));
            }
        }
    }
}
