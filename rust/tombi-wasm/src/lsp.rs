use futures::TryStreamExt;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::stream::JsStream;

/// Browser streams used to exchange LSP messages with a Web Worker.
#[wasm_bindgen]
pub struct ServerConfig {
    into_server: js_sys::AsyncIterator,
    from_server: web_sys::WritableStream,
}

#[wasm_bindgen]
impl ServerConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(into_server: js_sys::AsyncIterator, from_server: web_sys::WritableStream) -> Self {
        Self {
            into_server,
            from_server,
        }
    }
}

/// Serve Tombi LSP over byte streams using the standard LSP header framing.
#[wasm_bindgen]
pub async fn serve(config: ServerConfig) -> Result<(), JsValue> {
    let ServerConfig {
        into_server,
        from_server,
    } = config;

    let input = JsStream::from(into_server)
        .map_ok(|value| {
            value
                .dyn_into::<js_sys::Uint8Array>()
                .expect("LSP input items must be Uint8Array")
                .to_vec()
        })
        .map_err(|_| std::io::Error::other("failed to read LSP input stream"))
        .into_async_read();

    let output = from_server.unchecked_into::<wasm_streams::writable::sys::WritableStream>();
    let output = wasm_streams::WritableStream::from_raw(output)
        .try_into_async_write()
        .map_err(|error| error.0)?;

    let (service, socket) = tombi_lsp::lsp_service(false, false);
    tower_lsp::Server::new(input, output, socket)
        .serve(service)
        .await;

    Ok(())
}
