//! Shared format/lint core for Tombi's wasm-lib, Python and Node.js bindings.
//!
//! [`format_async`]/[`lint_async`] are the shared core: they resolve config and
//! schemas the same way regardless of target, so `tombi-wasm`'s `lib` feature
//! calls them directly from its own async (wasm-bindgen-futures) executor.
//! `format`/`lint` additionally block on a Tokio runtime, for synchronous
//! callers such as the Python (`python` feature) and Node.js (`node` feature)
//! bindings; they are only available on non-wasm targets.

mod error;

pub use error::Error;
pub use tombi_diagnostic::Diagnostic;

/// An in-memory `tombi.toml` configuration.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum ConfigInput {
    /// The content of a config file at a given path.
    File {
        content: String,
        path: std::path::PathBuf,
    },
    /// The content of a virtual `tombi.toml`.
    Text(String),
}

/// Options shared by `format`/[`format_async`] and `lint`/[`lint_async`].
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
    pub config: Option<ConfigInput>,
}

/// The result of formatting a TOML document.
#[derive(Debug, Clone)]
pub struct FormatResult {
    /// The formatted source, or `None` if formatting failed.
    pub formatted: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

/// The result of linting a TOML document.
#[derive(Debug, Clone, Default)]
pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
}

fn load_config(
    options: Options,
) -> Result<(tombi_config::Config, Option<std::path::PathBuf>), Error> {
    if let Some(config) = options.config {
        let (config_content, config_path) = match config {
            ConfigInput::File { content, path } => (content, path),
            ConfigInput::Text(content) => (
                content,
                std::path::PathBuf::from(tombi_config::TOMBI_TOML_FILENAME),
            ),
        };
        let config =
            serde_tombi::config::from_str(&config_content, &config_path).map_err(|error| {
                log::warn!("{error}");
                tombi_config::Error::ConfigFileParseFailed {
                    config_path: config_path.clone(),
                }
            })?;
        Ok((config, Some(config_path)))
    } else {
        Ok(serde_tombi::config::load_with_path(
            std::env::current_dir().ok(),
        )?)
    }
}

fn new_schema_store(config: &tombi_config::Config) -> tombi_schema_store::SchemaStore {
    let schema_options = config.schema.as_ref();
    // `offline`/`cache` are not yet exposed on `Options`: this issue only
    // establishes the shared core, and the disk cache/offline mode are only
    // meaningful for a real filesystem (the `native` feature), so exposing
    // them is deferred to the `python`/`node` binding work that actually
    // needs them (#2203/#2204).
    tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
        offline: None,
        strict: schema_options.and_then(|schema_options| schema_options.strict()),
        cache: None,
    })
}

/// Format a TOML document.
pub async fn format_async(
    source: String,
    source_path: String,
    options: Options,
) -> Result<FormatResult, Error> {
    let source_path = std::path::PathBuf::from(source_path);
    let (config, config_path) = load_config(options)?;
    let toml_version = config.toml_version.unwrap_or_default();
    let schema_store = new_schema_store(&config);

    schema_store
        .load_config(&config, config_path.as_deref())
        .await?;

    let Some(format_options) =
        tombi_glob::get_format_options(&config, Some(&source_path), config_path.as_deref())
    else {
        // If formatting is disabled, return the source as-is
        return Ok(FormatResult {
            formatted: Some(source),
            diagnostics: Vec::new(),
        });
    };

    match tombi_formatter::Formatter::new(
        toml_version,
        &format_options,
        Some(itertools::Either::Right(&source_path)),
        &schema_store,
    )
    .format(&source)
    .await
    {
        Ok(formatted) => Ok(FormatResult {
            formatted: Some(formatted),
            diagnostics: Vec::new(),
        }),
        Err(diagnostics) => Ok(FormatResult {
            formatted: None,
            diagnostics,
        }),
    }
}

/// Lint a TOML document.
pub async fn lint_async(
    source: String,
    source_path: String,
    options: Options,
) -> Result<LintResult, Error> {
    let source_path = std::path::PathBuf::from(source_path);
    let (config, config_path) = load_config(options)?;
    let toml_version = config.toml_version.unwrap_or_default();
    let schema_store = new_schema_store(&config);

    schema_store
        .load_config(&config, config_path.as_deref())
        .await?;

    let Some(lint_options) =
        tombi_glob::get_lint_options(&config, Some(&source_path), config_path.as_deref())
    else {
        // If linting is disabled, return success
        return Ok(LintResult::default());
    };

    match tombi_linter::Linter::new(
        toml_version,
        &lint_options,
        Some(itertools::Either::Right(&source_path)),
        &schema_store,
    )
    .lint(&source)
    .await
    {
        Ok(()) => Ok(LintResult::default()),
        Err(diagnostics) => Ok(LintResult { diagnostics }),
    }
}

#[cfg(not(target_family = "wasm"))]
fn runtime() -> Result<tokio::runtime::Runtime, Error> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?)
}

/// Format a TOML document, blocking on [`format_async`] internally.
///
/// Not available on wasm targets; use [`format_async`] instead.
#[cfg(not(target_family = "wasm"))]
pub fn format(
    source: String,
    source_path: String,
    options: Options,
) -> Result<FormatResult, Error> {
    runtime()?.block_on(format_async(source, source_path, options))
}

/// Lint a TOML document, blocking on [`lint_async`] internally.
///
/// Not available on wasm targets; use [`lint_async`] instead.
#[cfg(not(target_family = "wasm"))]
pub fn lint(source: String, source_path: String, options: Options) -> Result<LintResult, Error> {
    runtime()?.block_on(lint_async(source, source_path, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    // `Options::default()` (no explicit config) would make `load_config` walk
    // up from the real process cwd and pick up this repository's own
    // `tombi.toml` (which enables a network schema catalog), so every test
    // here passes an explicit config that disables schema resolution or
    // scopes it to a local file, keeping tests hermetic and network-free.
    fn schema_disabled_options() -> Options {
        Options {
            config: Some(ConfigInput::Text("[schema]\nenabled = false\n".to_owned())),
        }
    }

    #[test]
    fn format_returns_formatted_source() {
        let result = format(
            "key=1".to_owned(),
            "playground.toml".to_owned(),
            schema_disabled_options(),
        )
        .unwrap();

        assert_eq!(result.formatted.as_deref(), Some("key = 1\n"));
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn lint_reports_diagnostics_for_invalid_toml() {
        let result = lint(
            "key =".to_owned(),
            "playground.toml".to_owned(),
            schema_disabled_options(),
        )
        .unwrap();

        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn config_parse_failure_surfaces_as_config_error() {
        let error = lint(
            "key = 1".to_owned(),
            "playground.toml".to_owned(),
            Options {
                config: Some(ConfigInput::Text("invalid =".to_owned())),
            },
        )
        .unwrap_err();

        assert!(matches!(error, Error::Config(_)));
    }

    #[test]
    fn lint_resolves_local_file_schema_without_network_catalog() {
        let dir = std::env::temp_dir();
        let unique = format!(
            "{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let schema_path = dir.join(format!("tombi_lib_schema_{unique}.json"));
        let config_path = dir.join(format!("tombi_lib_config_{unique}.toml"));
        std::fs::write(
            &schema_path,
            r#"{"type":"object","properties":{"key":{"type":"integer"}}}"#,
        )
        .unwrap();

        // `schema.catalog.paths = []` disables the default (network) schema
        // catalog so this test does not depend on outbound network access.
        // `path` is relative to `config_path`'s directory, which does not
        // need to exist on disk (only the schema file does).
        let config = format!(
            r#"
[[schemas]]
path = "{}"
include = ["data.toml"]

[schema.catalog]
paths = []
"#,
            schema_path.file_name().unwrap().to_string_lossy(),
        );

        let violation = lint(
            r#"key = "not-an-integer""#.to_owned(),
            "data.toml".to_owned(),
            Options {
                config: Some(ConfigInput::File {
                    content: config.clone(),
                    path: config_path.clone(),
                }),
            },
        )
        .unwrap();
        assert!(!violation.diagnostics.is_empty());

        let compliant = lint(
            "key = 1".to_owned(),
            "data.toml".to_owned(),
            Options {
                config: Some(ConfigInput::File {
                    content: config,
                    path: config_path,
                }),
            },
        )
        .unwrap();
        assert!(compliant.diagnostics.is_empty());

        let _ = std::fs::remove_file(schema_path);
    }
}
