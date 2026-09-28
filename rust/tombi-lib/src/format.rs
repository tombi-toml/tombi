use crate::{Diagnostic, Error, Options};

/// The result of formatting a TOML document.
///
/// `Serialize` is only derived under the `wasm` feature: `Diagnostic` itself
/// only implements `Serialize` there (`tombi-diagnostic`'s `wasm` feature).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
pub struct FormatResult {
    /// The formatted source, or `None` if formatting failed.
    #[cfg_attr(feature = "wasm", serde(skip_serializing_if = "Option::is_none"))]
    pub formatted: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Format a TOML document.
pub async fn format_async(
    source: String,
    source_path: String,
    options: Options,
) -> Result<FormatResult, Error> {
    let source_path = std::path::PathBuf::from(source_path);
    let (config, config_path) = crate::load_config(options)?;
    let toml_version = config.toml_version.unwrap_or_default();
    let schema_store = crate::new_schema_store(&config);

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

/// Format a TOML document, blocking on [`format_async`] internally.
///
/// Not available on wasm targets; use [`format_async`] instead.
#[cfg(not(target_family = "wasm"))]
pub fn format_sync(
    source: String,
    source_path: String,
    options: Options,
) -> Result<FormatResult, Error> {
    crate::runtime()?.block_on(format_async(source, source_path, options))
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use crate::test_util::schema_disabled_options;

    // Declarative macro for this crate's own `format_sync` entry point, in
    // the spirit of `tombi_formatter::test_format!`. That macro can't be
    // reused directly: it builds a `Formatter` against a pre-built
    // `SchemaStore` fixture, bypassing exactly what this test exercises
    // (config loading, schema-store wiring, and `Error` mapping in
    // `load_config`/`new_schema_store`). Further same-shaped `format_sync`
    // cases should be added here rather than as new one-off `#[test]` fns.
    macro_rules! test_lib_format {
        ($name:ident, $source:expr, $options:expr => formatted: $expected:expr) => {
            #[test]
            fn $name() {
                let result =
                    format_sync($source.to_owned(), "playground.toml".to_owned(), $options)
                        .unwrap();
                assert_eq!(result.formatted.as_deref(), $expected);
            }
        };
    }

    test_lib_format!(
        format_returns_formatted_source,
        "key=1",
        schema_disabled_options() => formatted: Some("key = 1\n")
    );
}
