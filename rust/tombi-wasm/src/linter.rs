use js_sys::Promise;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::future_to_promise;

use crate::convert::{deserialize_options, serialize, tombi_error};

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
