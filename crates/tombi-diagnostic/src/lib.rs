mod level;
pub mod printer;

pub use level::Level;
pub use printer::Print;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    level: level::Level,
    message: String,
    range: tombi_text::Range,
    source_file: Option<std::path::PathBuf>,
}

impl Diagnostic {
    #[inline]
    pub fn new_warning(message: impl Into<String>, range: impl Into<tombi_text::Range>) -> Self {
        Self {
            level: level::Level::WARNING,
            message: message.into(),
            range: range.into(),
            source_file: None,
        }
    }

    #[inline]
    pub fn new_error(message: impl Into<String>, range: impl Into<tombi_text::Range>) -> Self {
        Self {
            level: level::Level::ERROR,
            message: message.into(),
            range: range.into(),
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
    pub fn message(&self) -> &str {
        &self.message
    }

    #[inline]
    pub fn position(&self) -> tombi_text::Position {
        self.range.start
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.range
    }

    #[inline]
    pub fn source_file(&self) -> Option<&std::path::Path> {
        self.source_file.as_deref()
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

impl From<Diagnostic> for tower_lsp::lsp_types::Diagnostic {
    fn from(diagnostic: Diagnostic) -> Self {
        tower_lsp::lsp_types::Diagnostic {
            range: diagnostic.range().into(),
            severity: Some(match diagnostic.level() {
                level::Level::WARNING => tower_lsp::lsp_types::DiagnosticSeverity::WARNING,
                level::Level::ERROR => tower_lsp::lsp_types::DiagnosticSeverity::ERROR,
            }),
            message: diagnostic.message().to_string(),
            ..Default::default()
        }
    }
}
