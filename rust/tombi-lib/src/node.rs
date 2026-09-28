//! napi-rs bindings for `@tombi-toml/tombi-lib`.
//!
//! `format`/`lint` return a `Promise`: the synchronous core
//! ([`crate::format_sync`]/[`crate::lint_sync`]) runs on the libuv thread pool
//! via [`AsyncTask`], so it never blocks the Node.js event loop.

use napi::{
    Env, Task,
    bindgen_prelude::{AsyncTask, Either, Null},
};
use napi_derive::napi;

/// The content of a `tombi.toml` config file at a given path.
#[napi(object, js_name = "ConfigFile")]
pub struct JsConfigFile {
    pub content: String,
    pub path: String,
}

/// Options shared by the formatter and linter.
#[napi(object, js_name = "Options")]
pub struct JsOptions {
    /// An in-memory `tombi.toml` configuration.
    /// When a string is provided, it is treated as the content of a virtual
    /// `tombi.toml`. When omitted, `tombi.toml` is searched from the current
    /// working directory.
    pub config: Option<Either<String, JsConfigFile>>,
}

impl From<JsOptions> for crate::Options {
    fn from(options: JsOptions) -> Self {
        Self {
            config: options.config.map(|config| match config {
                Either::A(content) => crate::ConfigInput::Text(content),
                Either::B(JsConfigFile { content, path }) => crate::ConfigInput::File {
                    content,
                    path: path.into(),
                },
            }),
        }
    }
}

/// A zero-based position in a TOML document.
#[napi(object, js_name = "Position")]
pub struct JsPosition {
    pub line: u32,
    pub column: u32,
}

impl From<tombi_text::Position> for JsPosition {
    fn from(position: tombi_text::Position) -> Self {
        Self {
            line: position.line,
            column: position.column,
        }
    }
}

/// A range in a TOML document.
#[napi(object, js_name = "Range")]
pub struct JsRange {
    pub start: JsPosition,
    pub end: JsPosition,
}

/// A diagnostic reported by Tombi.
#[napi(object, js_name = "Diagnostic")]
pub struct JsDiagnostic {
    #[napi(ts_type = "\"error\" | \"warning\"")]
    pub level: String,
    pub code: String,
    pub message: String,
    pub range: JsRange,
    // `snake_case` (and `null` rather than a missing key) to keep the same
    // shape as `@tombi-toml/wasm-lib`'s `Diagnostic`.
    #[napi(js_name = "source_file")]
    pub source_file: Either<String, Null>,
}

impl From<crate::Diagnostic> for JsDiagnostic {
    fn from(diagnostic: crate::Diagnostic) -> Self {
        let range = diagnostic.range();
        Self {
            level: match diagnostic.level() {
                tombi_diagnostic::Level::ERROR => "error",
                tombi_diagnostic::Level::WARNING => "warning",
            }
            .to_owned(),
            code: diagnostic.code().to_owned(),
            message: diagnostic.message().to_owned(),
            range: JsRange {
                start: range.start.into(),
                end: range.end.into(),
            },
            source_file: match diagnostic.source_file() {
                Some(source_file) => Either::A(source_file.to_string_lossy().into_owned()),
                None => Either::B(Null),
            },
        }
    }
}

/// The result of formatting a TOML document.
#[napi(object, js_name = "FormatResult")]
pub struct JsFormatResult {
    /// The formatted source, or `undefined` if formatting failed.
    pub formatted: Option<String>,
    pub diagnostics: Vec<JsDiagnostic>,
}

/// The result of linting a TOML document.
#[napi(object, js_name = "LintResult")]
pub struct JsLintResult {
    pub diagnostics: Vec<JsDiagnostic>,
}

fn to_napi_error(error: crate::Error) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}

pub struct FormatTask {
    source: String,
    source_path: String,
    options: crate::Options,
}

impl Task for FormatTask {
    type Output = crate::FormatResult;
    type JsValue = JsFormatResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        crate::format_sync(
            std::mem::take(&mut self.source),
            std::mem::take(&mut self.source_path),
            std::mem::take(&mut self.options),
        )
        .map_err(to_napi_error)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(JsFormatResult {
            formatted: output.formatted,
            diagnostics: output.diagnostics.into_iter().map(Into::into).collect(),
        })
    }
}

pub struct LintTask {
    source: String,
    source_path: String,
    options: crate::Options,
}

impl Task for LintTask {
    type Output = crate::LintResult;
    type JsValue = JsLintResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        crate::lint_sync(
            std::mem::take(&mut self.source),
            std::mem::take(&mut self.source_path),
            std::mem::take(&mut self.options),
        )
        .map_err(to_napi_error)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(JsLintResult {
            diagnostics: output.diagnostics.into_iter().map(Into::into).collect(),
        })
    }
}

/// Format a TOML document.
#[napi(ts_return_type = "Promise<FormatResult>")]
pub fn format(
    source: String,
    source_path: String,
    options: Option<JsOptions>,
) -> AsyncTask<FormatTask> {
    AsyncTask::new(FormatTask {
        source,
        source_path,
        options: options.map(Into::into).unwrap_or_default(),
    })
}

/// Lint a TOML document.
#[napi(ts_return_type = "Promise<LintResult>")]
pub fn lint(
    source: String,
    source_path: String,
    options: Option<JsOptions>,
) -> AsyncTask<LintTask> {
    AsyncTask::new(LintTask {
        source,
        source_path,
        options: options.map(Into::into).unwrap_or_default(),
    })
}
