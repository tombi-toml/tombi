use crate::{Diagnostic, Error, Options};

/// The result of linting a TOML document.
///
/// `Serialize` is only derived under the `wasm` feature: `Diagnostic` itself
/// only implements `Serialize` there.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, skip_from_py_object))]
pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
}

/// Lint a TOML document.
pub async fn lint_async(
    source: String,
    source_path: String,
    options: Options,
) -> Result<LintResult, Error> {
    let source_path = std::path::PathBuf::from(source_path);
    let (config, config_path) = crate::load_config(options)?;
    let toml_version = config.toml_version.unwrap_or_default();
    let schema_store = crate::new_schema_store(&config);

    schema_store
        .load_config(&config, config_path.as_deref())
        .await?;

    let Some(lint_options) =
        tombi_glob::get_lint_options(&config, Some(&source_path), config_path.as_deref())
    else {
        // If linting is disabled, return success
        return Ok(LintResult::default());
    };

    let parsed = tombi_parser::parse(&source);
    match tombi_linter::Linter::new(
        toml_version,
        &lint_options,
        Some(itertools::Either::Right(&source_path)),
        &schema_store,
    )
    .lint_parsed(&parsed)
    .await
    {
        Ok(()) => Ok(LintResult::default()),
        Err(diagnostics) => Ok(LintResult {
            diagnostics: Diagnostic::from_diagnostics(diagnostics, parsed.line_index()),
        }),
    }
}

/// Lint a TOML document, blocking on [`lint_async`] internally.
///
/// Not available on wasm targets; use [`lint_async`] instead.
#[cfg(not(target_family = "wasm"))]
pub fn lint_sync(
    source: String,
    source_path: String,
    options: Options,
) -> Result<LintResult, Error> {
    crate::runtime()?.block_on(lint_async(source, source_path, options))
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use crate::{ConfigInput, test_util::schema_disabled_options};

    // Declarative macro for this crate's own `lint_sync` entry point, in the
    // spirit of `tombi_linter::test_lint!`. That macro can't be reused
    // directly: it builds a `Linter` against a pre-built `SchemaStore`
    // fixture, bypassing exactly what this test exercises (config loading,
    // schema-store wiring, and `Error` mapping in `load_config`/
    // `new_schema_store`). Further same-shaped `lint_sync` cases should be
    // added here rather than as new one-off `#[test]` fns.
    macro_rules! test_lib_lint {
        ($name:ident, $source:expr, $options:expr => has_diagnostics: $expected:expr) => {
            #[test]
            fn $name() {
                let result =
                    lint_sync($source.to_owned(), "playground.toml".to_owned(), $options).unwrap();
                assert_eq!(!result.diagnostics.is_empty(), $expected);
            }
        };
    }

    test_lib_lint!(
        lint_reports_diagnostics_for_invalid_toml,
        "key =",
        schema_disabled_options() => has_diagnostics: true
    );

    #[test]
    fn config_parse_failure_surfaces_as_config_error() {
        let error = lint_sync(
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

        let violation = lint_sync(
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

        let compliant = lint_sync(
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
