use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

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
    tombi_error_from_message(&error.to_string())
}

fn tombi_error_from_message(message: &str) -> JsValue {
    let js_error = Error::new(message);
    js_error.set_name(tombi_lib::Error::NAME);
    js_error.into()
}

/// Run `future` to completion without an executor.
///
/// WASM can't block the JS thread, so the future is polled exactly once: it
/// completes when every input is available synchronously (the source, an
/// in-memory config, and schemas injected into the virtual filesystem), and
/// throws when it has to wait on asynchronous I/O such as fetching a remote
/// schema, which only `format`/`lint` can do.
fn run_sync<T>(future: impl Future<Output = Result<T, tombi_lib::Error>>) -> Result<T, JsValue> {
    match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(result) => result.map_err(tombi_error),
        Poll::Pending => Err(tombi_error_from_message(
            "formatSync/lintSync can't wait on asynchronous I/O (e.g. fetching a remote schema). \
             Use format/lint, or disable remote schemas in the configuration.",
        )),
    }
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

#[wasm_bindgen(js_name = formatSync)]
pub fn format_sync(
    source: String,
    source_path: String,
    options: JsValue,
) -> Result<JsValue, JsValue> {
    let options = deserialize_options(options)?;
    run_sync(tombi_lib::format_async(source, source_path, options)).map(|result| serialize(&result))
}

#[wasm_bindgen(js_name = lintSync)]
pub fn lint_sync(
    source: String,
    source_path: String,
    options: JsValue,
) -> Result<JsValue, JsValue> {
    let options = deserialize_options(options)?;
    run_sync(tombi_lib::lint_async(source, source_path, options)).map(|result| serialize(&result))
}
