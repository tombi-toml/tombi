//! napi-rs bindings for `@tombi-toml/lib`.
//!
//! `format`/`lint` return a `Promise`: the synchronous core
//! ([`tombi_lib::format_sync`]/[`tombi_lib::lint_sync`]) runs on the libuv
//! thread pool via [`AsyncTask`], so it never blocks the Node.js event loop.
//! `formatSync`/`lintSync` run the same core on the calling thread instead.

mod error;

use error::{Failure, to_napi_error};
use napi::{
    Env, Task,
    bindgen_prelude::{AsyncTask, Either, Null, Unknown},
};
use napi_derive::napi;

// `ConfigFile`/`Options` only declare the TypeScript types of `options`. The
// value itself is deserialized straight into `tombi_lib::Options` with serde,
// like wasm-lib, so unknown keys and non-object values are rejected the same
// way (napi's own object conversion would silently ignore them).

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

fn deserialize_options(
    env: Env,
    options: Option<Unknown<'_>>,
) -> Result<tombi_lib::Options, String> {
    match options {
        Some(options) => env
            .from_js_value::<tombi_lib::Options, _>(options)
            .map_err(|error| error.reason),
        None => Ok(tombi_lib::Options::default()),
    }
}

/// A zero-based position in a TOML document.
#[napi(object, js_name = "Position")]
pub struct JsPosition {
    pub line: u32,
    pub column: u32,
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
    // `null` rather than a missing key, to keep the same shape as
    // `@tombi-toml/wasm-lib`'s `Diagnostic`.
    pub source_file: Either<String, Null>,
}

impl From<tombi_lib::Diagnostic> for JsDiagnostic {
    fn from(diagnostic: tombi_lib::Diagnostic) -> Self {
        let range = diagnostic.range();
        Self {
            level: if diagnostic.is_error() {
                "error"
            } else {
                "warning"
            }
            .to_owned(),
            code: diagnostic.code().to_owned(),
            message: diagnostic.message().to_owned(),
            range: JsRange {
                start: JsPosition {
                    line: range.start.line,
                    column: range.start.column,
                },
                end: JsPosition {
                    line: range.end.line,
                    column: range.end.column,
                },
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

/// The arguments of one `format`/`lint` call, moved onto the libuv thread
/// pool by [`AsyncTask`].
pub struct Request {
    source: String,
    source_path: String,
    options: Result<tombi_lib::Options, String>,
}

impl Request {
    fn new(env: Env, source: String, source_path: String, options: Option<Unknown<'_>>) -> Self {
        Self {
            source,
            source_path,
            options: deserialize_options(env, options),
        }
    }

    fn run<T>(
        &mut self,
        run: fn(String, String, tombi_lib::Options) -> Result<T, tombi_lib::Error>,
    ) -> Result<T, Failure> {
        let options = std::mem::replace(&mut self.options, Ok(tombi_lib::Options::default()))
            .map_err(Failure::InvalidOptions)?;
        run(
            std::mem::take(&mut self.source),
            std::mem::take(&mut self.source_path),
            options,
        )
        .map_err(Failure::Tombi)
    }
}

fn format_result(output: tombi_lib::FormatResult) -> JsFormatResult {
    JsFormatResult {
        formatted: output.formatted,
        diagnostics: output.diagnostics.into_iter().map(Into::into).collect(),
    }
}

fn lint_result(output: tombi_lib::LintResult) -> JsLintResult {
    JsLintResult {
        diagnostics: output.diagnostics.into_iter().map(Into::into).collect(),
    }
}

pub struct FormatTask(Request);

impl Task for FormatTask {
    type Output = Result<tombi_lib::FormatResult, Failure>;
    type JsValue = JsFormatResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(self.0.run(tombi_lib::format_sync))
    }

    fn resolve(&mut self, env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        output
            .map(format_result)
            .map_err(|failure| to_napi_error(env, failure))
    }
}

pub struct LintTask(Request);

impl Task for LintTask {
    type Output = Result<tombi_lib::LintResult, Failure>;
    type JsValue = JsLintResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(self.0.run(tombi_lib::lint_sync))
    }

    fn resolve(&mut self, env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        output
            .map(lint_result)
            .map_err(|failure| to_napi_error(env, failure))
    }
}

/// Format a TOML document.
#[napi(
    ts_args_type = "source: string, sourcePath: string, options?: Options | undefined | null",
    ts_return_type = "Promise<FormatResult>"
)]
pub fn format(
    env: Env,
    source: String,
    source_path: String,
    options: Option<Unknown<'_>>,
) -> AsyncTask<FormatTask> {
    AsyncTask::new(FormatTask(Request::new(env, source, source_path, options)))
}

/// Lint a TOML document.
#[napi(
    ts_args_type = "source: string, sourcePath: string, options?: Options | undefined | null",
    ts_return_type = "Promise<LintResult>"
)]
pub fn lint(
    env: Env,
    source: String,
    source_path: String,
    options: Option<Unknown<'_>>,
) -> AsyncTask<LintTask> {
    AsyncTask::new(LintTask(Request::new(env, source, source_path, options)))
}

/// Format a TOML document synchronously, blocking the calling thread.
#[napi(ts_args_type = "source: string, sourcePath: string, options?: Options | undefined | null")]
pub fn format_sync(
    env: Env,
    source: String,
    source_path: String,
    options: Option<Unknown<'_>>,
) -> napi::Result<JsFormatResult> {
    Request::new(env, source, source_path, options)
        .run(tombi_lib::format_sync)
        .map(format_result)
        .map_err(|failure| to_napi_error(env, failure))
}

/// Lint a TOML document synchronously, blocking the calling thread.
#[napi(ts_args_type = "source: string, sourcePath: string, options?: Options | undefined | null")]
pub fn lint_sync(
    env: Env,
    source: String,
    source_path: String,
    options: Option<Unknown<'_>>,
) -> napi::Result<JsLintResult> {
    Request::new(env, source, source_path, options)
        .run(tombi_lib::lint_sync)
        .map(lint_result)
        .map_err(|failure| to_napi_error(env, failure))
}
