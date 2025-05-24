mod format;
mod pretty_buf;

use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn format(file_name: String, content: String) -> Promise {
    let fut = format::format(std::path::PathBuf::from(file_name), content);

    future_to_promise(async move {
        match fut.await {
            Ok(t) => Ok(JsValue::from_str(&t)),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    })
}
