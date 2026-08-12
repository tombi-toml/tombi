use js_sys::{Object, Promise, Reflect};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use tombi_diagnostic::Diagnostic;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::future_to_promise;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    config: Option<ConfigInput>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ConfigInput {
    File { context: String, path: String },
    Text(String),
}

enum FormatResult {
    Formatted(String),
    Diagnostics(Vec<Diagnostic>),
}

fn serialize(value: &impl Serialize) -> JsValue {
    value
        .serialize(&Serializer::json_compatible())
        .expect("WASM values must be serializable")
}

fn serialize_format_result(result: FormatResult) -> JsValue {
    let object = Object::new();
    let (formatted, diagnostics) = match result {
        FormatResult::Formatted(formatted) => (JsValue::from_str(&formatted), JsValue::UNDEFINED),
        FormatResult::Diagnostics(diagnostics) => (JsValue::UNDEFINED, serialize(&diagnostics)),
    };

    Reflect::set(&object, &JsValue::from_str("formatted"), &formatted)
        .expect("WASM format results must be writable");
    Reflect::set(&object, &JsValue::from_str("diagnostics"), &diagnostics)
        .expect("WASM format results must be writable");
    object.into()
}

fn deserialize_options(options: JsValue) -> Result<Options, String> {
    if options.is_null() || options.is_undefined() {
        Ok(Options::default())
    } else {
        serde_wasm_bindgen::from_value(options).map_err(|error| error.to_string())
    }
}

fn load_config(
    options: Options,
    source_path: &std::path::Path,
) -> Result<(tombi_config::Config, Option<std::path::PathBuf>), String> {
    if let Some(config) = options.config {
        let (config_context, config_path) = match config {
            ConfigInput::File { context, path } => (context, std::path::PathBuf::from(path)),
            ConfigInput::Text(context) => (
                context,
                source_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(""))
                    .join(tombi_config::TOMBI_TOML_FILENAME),
            ),
        };
        let config = serde_tombi::config::from_str(&config_context, &config_path)
            .map_err(|error| error.to_string())?;
        Ok((config, Some(config_path)))
    } else {
        serde_tombi::config::load_with_path(std::env::current_dir().ok())
            .map_err(|error| error.to_string())
    }
}

#[wasm_bindgen]
pub fn format(source: String, source_path: String, options: JsValue) -> Promise {
    #[derive(serde::Serialize, Debug)]
    struct FormatError {
        error: String,
    }

    async fn inner_format(
        source: String,
        source_path: String,
        options: JsValue,
    ) -> Result<FormatResult, FormatError> {
        let source_path = std::path::PathBuf::from(source_path);
        let options = deserialize_options(options).map_err(|error| FormatError { error })?;
        let (config, config_path) =
            load_config(options, &source_path).map_err(|error| FormatError { error })?;
        let toml_version = config.toml_version.unwrap_or_default();

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
            return Err(FormatError { error });
        }

        // Get format options with override support
        let Some(format_options) =
            tombi_glob::get_format_options(&config, Some(&source_path), config_path.as_deref())
        else {
            // If formatting is disabled, return the source as-is
            return Ok(FormatResult::Formatted(source));
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
            Ok(formatted) => Ok(FormatResult::Formatted(formatted)),
            Err(diagnostics) => Ok(FormatResult::Diagnostics(diagnostics)),
        }
    }

    future_to_promise(async move {
        match inner_format(source, source_path, options).await {
            Ok(result) => Ok(serialize_format_result(result)),
            Err(error) => Err(serialize(&error)),
        }
    })
}

#[wasm_bindgen]
pub fn lint(source: String, source_path: String, options: JsValue) -> Promise {
    #[derive(serde::Serialize, Debug)]
    struct LintResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        diagnostics: Option<Vec<Diagnostic>>,
    }

    #[derive(serde::Serialize, Debug)]
    struct LintError {
        error: String,
    }

    async fn inner_lint(
        source: String,
        source_path: String,
        options: JsValue,
    ) -> Result<LintResult, LintError> {
        let source_path = std::path::PathBuf::from(source_path);
        let options = deserialize_options(options).map_err(|error| LintError { error })?;
        let (config, config_path) =
            load_config(options, &source_path).map_err(|error| LintError { error })?;
        let toml_version = config.toml_version.unwrap_or_default();

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
            return Err(LintError { error });
        }

        // Get lint options with override support
        let Some(lint_options) =
            tombi_glob::get_lint_options(&config, Some(&source_path), config_path.as_deref())
        else {
            // If linting is disabled, return success
            return Ok(LintResult { diagnostics: None });
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
            Ok(_) => Ok(LintResult { diagnostics: None }),
            Err(diagnostics) => Ok(LintResult {
                diagnostics: Some(diagnostics),
            }),
        }
    }

    future_to_promise(async move {
        match inner_lint(source, source_path, options).await {
            Ok(result) => Ok(serialize(&result)),
            Err(error) => Err(serialize(&error)),
        }
    })
}
