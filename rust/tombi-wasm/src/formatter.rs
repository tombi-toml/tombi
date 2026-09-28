use js_sys::{Error, Promise, TypeError};
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::future_to_promise;

fn serialize(value: &impl Serialize) -> JsValue {
    value
        .serialize(&Serializer::json_compatible())
        .expect("WASM values must be serializable")
}

/// A JS `Error` named [`tombi_lib::Error::NAME`], shared with the
/// Python/Node.js bindings.
fn tombi_error(error: tombi_lib::Error) -> JsValue {
    let js_error = Error::new(&error.to_string());
    js_error.set_name(tombi_lib::Error::NAME);
    js_error.into()
}

fn deserialize_options(options: JsValue) -> Result<tombi_lib::Options, JsValue> {
    if options.is_null() || options.is_undefined() {
        Ok(tombi_lib::Options::default())
    } else {
        // Malformed options are a caller bug (not a `TombiError`), so they
        // reject with the standard `TypeError`, like the other bindings.
        // `serde_wasm_bindgen` only reads a struct's known fields, which would
        // skip `deny_unknown_fields`, so go through a `serde_json::Value`.
        serde_wasm_bindgen::from_value::<serde_json::Value>(options)
            .map_err(|error| error.to_string())
            .and_then(|options| serde_json::from_value(options).map_err(|error| error.to_string()))
            .map_err(|message| TypeError::new(&message).into())
    }
}

#[wasm_bindgen]
pub fn format(source: String, source_path: String, options: JsValue) -> Promise {
    future_to_promise(async move {
        let options = deserialize_options(options)?;
        match tombi_lib::format_async(source, source_path, options).await {
            Ok(result) => Ok(serialize(&result)),
            Err(error) => Err(tombi_error(error)),
        }
    })
}

#[wasm_bindgen]
pub fn lint(source: String, source_path: String, options: JsValue) -> Promise {
    future_to_promise(async move {
        let options = deserialize_options(options)?;
        match tombi_lib::lint_async(source, source_path, options).await {
            Ok(result) => Ok(serialize(&result)),
            Err(error) => Err(tombi_error(error)),
        }
    })
}
