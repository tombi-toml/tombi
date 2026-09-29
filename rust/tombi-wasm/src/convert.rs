//! Conversions between JS values and `tombi_lib` types, shared by
//! [`crate::format`] and [`crate::lint`].

use js_sys::{Error, TypeError};
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::JsValue;

/// Serialize a `format`/`lint` result, adding each diagnostic's deprecated
/// `source_file` key (the pre-`sourceFile` name) with the same value.
/// Remove the alias in the next major release.
pub(crate) fn serialize(value: &impl Serialize) -> JsValue {
    let mut value = serde_json::to_value(value).expect("WASM values must be serializable");
    if let Some(diagnostics) = value
        .get_mut("diagnostics")
        .and_then(serde_json::Value::as_array_mut)
    {
        for diagnostic in diagnostics
            .iter_mut()
            .filter_map(serde_json::Value::as_object_mut)
        {
            let source_file = diagnostic
                .get("sourceFile")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            diagnostic.insert("source_file".to_owned(), source_file);
        }
    }
    value
        .serialize(&Serializer::json_compatible())
        .expect("WASM values must be serializable")
}

/// A JS `Error` named [`tombi_lib::Error::NAME`], shared with the
/// Python/Node.js bindings.
pub(crate) fn tombi_error(error: tombi_lib::Error) -> JsValue {
    let js_error = Error::new(&error.to_string());
    js_error.set_name(tombi_lib::Error::NAME);
    js_error.into()
}

pub(crate) fn deserialize_options(options: JsValue) -> Result<tombi_lib::Options, JsValue> {
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
