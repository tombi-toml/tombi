mod level;
pub mod printer;

pub use level::Level;
pub use printer::Print;

/// A diagnostic of a source text.
///
/// The location is kept as a [`tombi_text::Span`], and converted with the
/// [`tombi_text::LineIndex`] of the source only when it is reported,
/// in the column unit the receiver needs.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    level: level::Level,
    code: String,
    message: String,
    span: tombi_text::Span,
    source_file: Option<std::path::PathBuf>,
}

impl Diagnostic {
    #[inline]
    pub fn new_warning(
        message: impl Into<String>,
        code: impl Into<String>,
        span: impl Into<tombi_text::Span>,
    ) -> Self {
        Self {
            level: level::Level::WARNING,
            code: code.into(),
            message: message.into(),
            span: span.into(),
            source_file: None,
        }
    }

    #[inline]
    pub fn new_error(
        message: impl Into<String>,
        code: impl Into<String>,
        span: impl Into<tombi_text::Span>,
    ) -> Self {
        Self {
            level: level::Level::ERROR,
            code: code.into(),
            message: message.into(),
            span: span.into(),
            source_file: None,
        }
    }

    pub fn with_source_file(mut self, source_file: impl Into<std::path::PathBuf>) -> Self {
        self.source_file = Some(source_file.into());
        self
    }

    #[inline]
    pub fn level(&self) -> level::Level {
        self.level
    }

    #[inline]
    pub fn is_warning(&self) -> bool {
        self.level == level::Level::WARNING
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        self.level == level::Level::ERROR
    }

    #[inline]
    pub fn code(&self) -> &str {
        &self.code
    }

    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.span
    }

    /// Converts the span into a range whose columns are counted in `encoding`.
    #[inline]
    pub fn range(
        &self,
        line_index: &tombi_text::LineIndex,
        encoding: tombi_text::EncodingKind,
    ) -> tombi_text::Range {
        line_index.range(self.span, encoding)
    }

    /// Pairs the diagnostic with a range already converted from its span.
    #[inline]
    pub fn with_range(&self, range: tombi_text::Range) -> LocatedDiagnostic<'_> {
        LocatedDiagnostic {
            diagnostic: self,
            range,
        }
    }

    /// Pairs the diagnostic with its range in the source indexed by `line_index`,
    /// whose columns are counted in `encoding`.
    pub fn located<'a>(
        &'a self,
        line_index: &tombi_text::LineIndex,
        encoding: tombi_text::EncodingKind,
    ) -> LocatedDiagnostic<'a> {
        LocatedDiagnostic {
            diagnostic: self,
            range: self.range(line_index, encoding),
        }
    }

    #[inline]
    pub fn source_file(&self) -> Option<&std::path::Path> {
        self.source_file.as_deref()
    }
}

/// A diagnostic with its range converted for a receiver, such as a printer.
#[derive(Debug, Clone, Copy)]
pub struct LocatedDiagnostic<'a> {
    diagnostic: &'a Diagnostic,
    range: tombi_text::Range,
}

impl LocatedDiagnostic<'_> {
    #[inline]
    pub fn diagnostic(&self) -> &Diagnostic {
        self.diagnostic
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl PartialEq for Diagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span && self.code == other.code && self.message == other.message
    }
}

impl Eq for Diagnostic {}

impl std::hash::Hash for Diagnostic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.code.hash(state);
        self.message.hash(state);
    }
}

pub trait SetDiagnostics {
    /// Set the diagnostic to the given diagnostics.
    ///
    /// We use set_diagnostic instead of to_diagnostic because self may have multiple diagnostics.
    fn set_diagnostics(self, diagnostics: &mut Vec<Diagnostic>);
}

impl<T: SetDiagnostics> SetDiagnostics for Vec<T> {
    fn set_diagnostics(self, diagnostics: &mut Vec<Diagnostic>) {
        for item in self {
            item.set_diagnostics(diagnostics);
        }
    }
}

#[cfg(feature = "lsp")]
impl tombi_text::FromLsp<Diagnostic> for tower_lsp::lsp_types::Diagnostic {
    fn from_lsp(
        source: Diagnostic,
        line_index: &tombi_text::LineIndex,
        encoding: tombi_text::EncodingKind,
    ) -> tower_lsp::lsp_types::Diagnostic {
        use tombi_text::IntoLsp;

        tower_lsp::lsp_types::Diagnostic {
            range: source.span.into_lsp(line_index, encoding),
            severity: Some(match source.level() {
                level::Level::WARNING => tower_lsp::lsp_types::DiagnosticSeverity::WARNING,
                level::Level::ERROR => tower_lsp::lsp_types::DiagnosticSeverity::ERROR,
            }),
            message: source.message().to_string(),
            source: Some("Tombi".to_owned()),
            code: Some(tower_lsp::lsp_types::NumberOrString::String(source.code)),
            ..Default::default()
        }
    }
}
