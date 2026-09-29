mod level;
pub mod printer;

pub use level::Level;
pub use printer::Print;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "python", pyo3::pyclass(skip_from_py_object))]
pub struct Diagnostic {
    level: level::Level,
    code: String,
    message: String,
    range: tombi_text::Range,
    source_file: Option<std::path::PathBuf>,
}

impl Diagnostic {
    #[inline]
    pub fn new_warning(
        message: impl Into<String>,
        code: impl Into<String>,
        range: impl Into<tombi_text::Range>,
    ) -> Self {
        Self {
            level: level::Level::WARNING,
            code: code.into(),
            message: message.into(),
            range: range.into(),
            source_file: None,
        }
    }

    #[inline]
    pub fn new_error(
        message: impl Into<String>,
        code: impl Into<String>,
        range: impl Into<tombi_text::Range>,
    ) -> Self {
        Self {
            level: level::Level::ERROR,
            code: code.into(),
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

impl PartialEq for Diagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range && self.code == other.code && self.message == other.message
    }
}

impl Eq for Diagnostic {}

impl std::hash::Hash for Diagnostic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.range.hash(state);
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

/// A zero-based position in a TOML document, exposed to Python with
/// dot-chain access (`diagnostic.range.start.line`). A Python-only
/// counterpart to `tombi_text::Position`, kept in this crate rather than
/// adding a `python` feature to `tombi-text` itself.
#[cfg(feature = "python")]
#[derive(Debug, Clone, Copy)]
#[pyo3::pyclass(get_all, skip_from_py_object)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "python")]
#[pyo3::pymethods]
impl Position {
    fn __repr__(&self) -> String {
        format!("Position(line={}, column={})", self.line, self.column)
    }
}

#[cfg(feature = "python")]
impl From<tombi_text::Position> for Position {
    fn from(position: tombi_text::Position) -> Self {
        Self {
            line: position.line,
            column: position.column,
        }
    }
}

/// A range in a TOML document, exposed to Python with dot-chain access
/// (`diagnostic.range.start`/`.end`). See [`Position`] for why this isn't
/// `tombi_text::Range` directly.
#[cfg(feature = "python")]
#[derive(Debug, Clone, Copy)]
#[pyo3::pyclass(get_all, skip_from_py_object)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[cfg(feature = "python")]
#[pyo3::pymethods]
impl Range {
    fn __repr__(&self) -> String {
        format!("Range(start={:?}, end={:?})", self.start, self.end)
    }
}

#[cfg(feature = "python")]
impl From<tombi_text::Range> for Range {
    fn from(range: tombi_text::Range) -> Self {
        Self {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

#[cfg(feature = "python")]
#[pyo3::pymethods]
impl Diagnostic {
    #[getter(level)]
    fn py_level(&self) -> &str {
        match self.level {
            level::Level::WARNING => "warning",
            level::Level::ERROR => "error",
        }
    }

    #[getter(code)]
    fn py_code(&self) -> &str {
        self.code()
    }

    #[getter(message)]
    fn py_message(&self) -> &str {
        self.message()
    }

    #[getter(range)]
    fn py_range(&self) -> Range {
        self.range.into()
    }

    #[getter(source_file)]
    fn py_source_file(&self) -> Option<String> {
        self.source_file()
            .map(|source_file| source_file.to_string_lossy().into_owned())
    }

    fn __repr__(&self) -> String {
        format!(
            "Diagnostic(level={:?}, code={:?}, message={:?})",
            self.py_level(),
            self.code(),
            self.message()
        )
    }
}

#[cfg(feature = "lsp")]
impl tombi_text::FromLsp<Diagnostic> for tower_lsp::lsp_types::Diagnostic {
    fn from_lsp(
        source: Diagnostic,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::Diagnostic {
        use tombi_text::IntoLsp;

        tower_lsp::lsp_types::Diagnostic {
            range: source.range().into_lsp(line_index),
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
