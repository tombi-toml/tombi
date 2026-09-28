use js_sys::{Error, Promise};
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::future_to_promise;

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

fn deserialize_options(options: JsValue) -> Result<tombi_lib::Options, JsValue> {
    if options.is_null() || options.is_undefined() {
        Ok(tombi_lib::Options::default())
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|error| tombi_wasm_error(&error.to_string()))
    }
}

#[wasm_bindgen]
pub fn format(source: String, source_path: String, options: JsValue) -> Promise {
    future_to_promise(async move {
        let options = deserialize_options(options)?;
        match tombi_lib::format_async(source, source_path, options).await {
            Ok(result) => Ok(serialize(&result)),
            Err(error) => Err(tombi_wasm_error(&error.to_string())),
        }
    })
}

#[wasm_bindgen]
pub fn lint(source: String, source_path: String, options: JsValue) -> Promise {
    future_to_promise(async move {
        let options = deserialize_options(options)?;
        match tombi_lib::lint_async(source, source_path, options).await {
            Ok(result) => Ok(serialize(&result)),
            Err(error) => Err(tombi_wasm_error(&error.to_string())),
        }
    })
}
