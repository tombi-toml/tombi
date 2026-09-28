use js_sys::{Error, Promise};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use tombi_lib::Diagnostic;
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

impl From<Options> for tombi_lib::Options {
    fn from(options: Options) -> Self {
        Self {
            config: options.config.map(Into::into),
        }
    }
}

impl From<ConfigInput> for tombi_lib::ConfigInput {
    fn from(config: ConfigInput) -> Self {
        match config {
            ConfigInput::File { content, path } => Self::File {
                content,
                path: std::path::PathBuf::from(path),
            },
            ConfigInput::Text(content) => Self::Text(content),
        }
    }
}

#[derive(Serialize)]
struct FormatResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    formatted: Option<String>,
    diagnostics: Vec<Diagnostic>,
}

impl From<tombi_lib::FormatResult> for FormatResult {
    fn from(result: tombi_lib::FormatResult) -> Self {
        Self {
            formatted: result.formatted,
            diagnostics: result.diagnostics,
        }
    }
}

#[derive(Serialize)]
struct LintResult {
    diagnostics: Vec<Diagnostic>,
}

impl From<tombi_lib::LintResult> for LintResult {
    fn from(result: tombi_lib::LintResult) -> Self {
        Self {
            diagnostics: result.diagnostics,
        }
    }
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

fn deserialize_options(options: JsValue) -> Result<Options, JsValue> {
    if options.is_null() || options.is_undefined() {
        Ok(Options::default())
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|error| tombi_wasm_error(&error.to_string()))
    }
}

#[wasm_bindgen]
pub fn format(source: String, source_path: String, options: JsValue) -> Promise {
    future_to_promise(async move {
        let options = deserialize_options(options)?;
        match tombi_lib::format_async(source, source_path, options.into()).await {
            Ok(result) => Ok(serialize(&FormatResult::from(result))),
            Err(error) => Err(tombi_wasm_error(&error.to_string())),
        }
    })
}

#[wasm_bindgen]
pub fn lint(source: String, source_path: String, options: JsValue) -> Promise {
    future_to_promise(async move {
        let options = deserialize_options(options)?;
        match tombi_lib::lint_async(source, source_path, options.into()).await {
            Ok(result) => Ok(serialize(&LintResult::from(result))),
            Err(error) => Err(tombi_wasm_error(&error.to_string())),
        }
    })
}
