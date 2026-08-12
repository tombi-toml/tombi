use js_sys::{Error, Object, Promise, Reflect};
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
    File { content: String, path: String },
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

fn tombi_wasm_error(message: &str) -> JsValue {
    let error = Error::new(message);
    error.set_name("TombiWasmError");
    error.into()
}

fn serialize_format_result(result: FormatResult) -> JsValue {
    let object = Object::new();
    let diagnostics = match result {
        FormatResult::Formatted(formatted) => {
            Reflect::set(
                &object,
                &JsValue::from_str("formatted"),
                &JsValue::from_str(&formatted),
            )
            .expect("WASM format results must be writable");
            serialize(&Vec::<Diagnostic>::new())
        }
        FormatResult::Diagnostics(diagnostics) => serialize(&diagnostics),
    };

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
) -> Result<(tombi_config::Config, Option<std::path::PathBuf>), String> {
    if let Some(config) = options.config {
        let (config_content, config_path) = match config {
            ConfigInput::File { content, path } => (content, std::path::PathBuf::from(path)),
            ConfigInput::Text(content) => (
                content,
                std::path::PathBuf::from(tombi_config::TOMBI_TOML_FILENAME),
            ),
        };
        let config = serde_tombi::config::from_str(&config_content, &config_path)
            .map_err(|error| error.to_string())?;
        Ok((config, Some(config_path)))
    } else {
        serde_tombi::config::load_with_path(std::env::current_dir().ok())
            .map_err(|error| error.to_string())
    }
}

#[wasm_bindgen]
pub fn format(source: String, source_path: String, options: JsValue) -> Promise {
    async fn inner_format(
        source: String,
        source_path: String,
        options: JsValue,
    ) -> Result<FormatResult, String> {
        let source_path = std::path::PathBuf::from(source_path);
        let options = deserialize_options(options)?;
        let (config, config_path) = load_config(options)?;
        let toml_version = config.toml_version.unwrap_or_default();

        let schema_options = config.schema.as_ref();
        let schema_store =
            tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
                offline: None,
                strict: schema_options.and_then(|schema_options| schema_options.strict()),
                cache: None,
            });

        schema_store
            .load_config(&config, config_path.as_deref())
            .await
            .map_err(|error| error.to_string())?;

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
            Err(error) => Err(tombi_wasm_error(&error)),
        }
    })
}

#[wasm_bindgen]
pub fn lint(source: String, source_path: String, options: JsValue) -> Promise {
    #[derive(serde::Serialize, Debug)]
    struct LintResult {
        diagnostics: Vec<Diagnostic>,
    }

    async fn inner_lint(
        source: String,
        source_path: String,
        options: JsValue,
    ) -> Result<LintResult, String> {
        let source_path = std::path::PathBuf::from(source_path);
        let options = deserialize_options(options)?;
        let (config, config_path) = load_config(options)?;
        let toml_version = config.toml_version.unwrap_or_default();

        let schema_options = config.schema.as_ref();
        let schema_store =
            tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
                offline: None,
                strict: schema_options.and_then(|schema_options| schema_options.strict()),
                cache: None,
            });

        schema_store
            .load_config(&config, config_path.as_deref())
            .await
            .map_err(|error| error.to_string())?;

        // Get lint options with override support
        let Some(lint_options) =
            tombi_glob::get_lint_options(&config, Some(&source_path), config_path.as_deref())
        else {
            // If linting is disabled, return success
            return Ok(LintResult {
                diagnostics: Vec::new(),
            });
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
            Ok(_) => Ok(LintResult {
                diagnostics: Vec::new(),
            }),
            Err(diagnostics) => Ok(LintResult { diagnostics }),
        }
    }

    future_to_promise(async move {
        match inner_lint(source, source_path, options).await {
            Ok(result) => Ok(serialize(&result)),
            Err(error) => Err(tombi_wasm_error(&error)),
        }
    })
}
