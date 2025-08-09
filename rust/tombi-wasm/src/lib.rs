use std::str::FromStr;

use js_sys::Promise;
use serde_wasm_bindgen;
use tombi_config::TomlVersion;
use tombi_diagnostic::Diagnostic;
use tombi_formatter::FormatDefinitions;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn format(source: String, file_path: Option<String>, toml_version: Option<String>) -> Promise {
    #[derive(serde::Serialize, Debug)]
    #[serde(untagged)]
    #[serde(rename_all = "camelCase")]
    enum FormatError {
        Error { error: String },
        Diagnostics { diagnostics: Vec<Diagnostic> },
    }

    async fn inner_format(
        source: String,
        file_path: Option<String>,
        toml_version: Option<String>,
    ) -> Result<String, FormatError> {
        let toml_version = match toml_version {
            Some(v) => match TomlVersion::from_str(&v) {
                Ok(v) => v,
                Err(_) => {
                    return Err(FormatError::Error {
                        error: "Invalid TOML version".to_string(),
                    });
                }
            },
            None => TomlVersion::default(),
        };

        let (config, config_path) =
            match serde_tombi::config::load_with_path(std::env::current_dir().ok()) {
                Ok((config, config_path)) => (config, config_path),
                Err(err) => {
                    return Err(FormatError::Error {
                        error: err.to_string(),
                    });
                }
            };

        let schema_options = config.schema.as_ref();
        let schema_store =
            tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
                offline: None,
                strict: schema_options.and_then(|schema_options| schema_options.strict()),
                cache: None,
            });

        if let Err(error) = schema_store
            .load_config(&config, config_path.as_deref())
            .await
            .map_err(|e| e.to_string())
        {
            return Err(FormatError::Error { error });
        }

        let format_options = config.format.clone().unwrap_or_default();
        let format_definitions = FormatDefinitions::default();

        let file_path_buf = file_path.map(|path| std::path::PathBuf::from(path));
        match tombi_formatter::Formatter::new(
            toml_version,
            &format_definitions,
            &format_options,
            file_path_buf
                .as_deref()
                .map(|path| itertools::Either::Right(path)),
            &schema_store,
        )
        .format(&source)
        .await
        {
            Ok(t) => Ok(t),
            Err(diagnostics) => Err(FormatError::Diagnostics { diagnostics }),
        }
    }

    future_to_promise(async move {
        match inner_format(source, file_path, toml_version).await {
            Ok(t) => Ok(JsValue::from_str(&t)),
            Err(e) => Err(serde_wasm_bindgen::to_value(&e).unwrap()),
        }
    })
}

#[wasm_bindgen]
pub fn lint(source: String, file_path: Option<String>, toml_version: Option<String>) -> Promise {
    #[derive(serde::Serialize, Debug)]
    #[serde(untagged)]
    #[serde(rename_all = "camelCase")]
    enum LintError {
        Error { error: String },
        Diagnostics { diagnostics: Vec<Diagnostic> },
    }

    async fn inner_lint(
        source: String,
        file_path: Option<String>,
        toml_version: Option<String>,
    ) -> Result<(), LintError> {
        let toml_version = match toml_version {
            Some(v) => match TomlVersion::from_str(&v) {
                Ok(v) => v,
                Err(_) => {
                    return Err(LintError::Error {
                        error: "Invalid TOML version".to_string(),
                    });
                }
            },
            None => TomlVersion::default(),
        };

        let (config, config_path) =
            match serde_tombi::config::load_with_path(std::env::current_dir().ok()) {
                Ok((config, config_path)) => (config, config_path),
                Err(err) => {
                    return Err(LintError::Error {
                        error: err.to_string(),
                    });
                }
            };

        let schema_options = config.schema.as_ref();
        let schema_store =
            tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
                offline: None,
                strict: schema_options.and_then(|schema_options| schema_options.strict()),
                cache: None,
            });

        if let Err(error) = schema_store
            .load_config(&config, config_path.as_deref())
            .await
            .map_err(|e| e.to_string())
        {
            return Err(LintError::Error { error });
        }

        let lint_options = config.lint.clone().unwrap_or_default();

        let file_path_buf = file_path.map(|path| std::path::PathBuf::from(path));
        match tombi_linter::Linter::new(
            toml_version,
            &lint_options,
            file_path_buf
                .as_deref()
                .map(|path| itertools::Either::Right(path)),
            &schema_store,
        )
        .lint(&source)
        .await
        {
            Ok(_) => Ok(()),
            Err(diagnostics) => Err(LintError::Diagnostics { diagnostics }),
        }
    }

    future_to_promise(async move {
        match inner_lint(source, file_path, toml_version).await {
            Ok(_) => Ok(JsValue::NULL),
            Err(e) => Err(serde_wasm_bindgen::to_value(&e).unwrap()),
        }
    })
}
