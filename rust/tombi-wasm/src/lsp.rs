use futures::TryStreamExt;
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::stream::JsStream;

/// Browser streams used to exchange LSP messages with a Web Worker.
#[wasm_bindgen]
pub struct ServerConfig {
    into_server: js_sys::AsyncIterator,
    from_server: web_sys::WritableStream,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceEntry {
    uri: String,
    #[serde(default)]
    kind: WorkspaceEntryKind,
    #[serde(default)]
    text: String,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
enum WorkspaceEntryKind {
    #[default]
    File,
    Directory,
}

fn workspace_path(uri: &str) -> Result<std::path::PathBuf, JsValue> {
    uri.parse::<tombi_uri::Uri>()
        .map_err(|error| JsValue::from_str(&format!("invalid workspace URI: {error}")))?
        .to_file_path()
        .map_err(|_| JsValue::from_str("workspace URI must be a file URI"))
}

/// Update one file in the browser-backed virtual workspace.
#[wasm_bindgen]
pub fn set_workspace_file(uri: String, text: String) -> Result<(), JsValue> {
    tombi_fs::set_file(workspace_path(&uri)?, text);
    Ok(())
}

/// Remove one file from the browser-backed virtual workspace.
#[wasm_bindgen]
pub fn remove_workspace_file(uri: String) -> Result<(), JsValue> {
    tombi_fs::remove_file(&workspace_path(&uri)?);
    Ok(())
}

/// Replace the virtual workspace files visible to the WASM language server.
#[wasm_bindgen]
pub fn set_workspace_files(files: JsValue) -> Result<(), JsValue> {
    set_workspace_entries(files)
}

/// Replace the virtual workspace entries visible to the WASM language server.
#[wasm_bindgen]
pub fn set_workspace_entries(entries: JsValue) -> Result<(), JsValue> {
    let entries: Vec<WorkspaceEntry> = serde_wasm_bindgen::from_value(entries)?;
    let mut parsed_entries = Vec::with_capacity(entries.len());
    for entry in entries {
        let path = workspace_path(&entry.uri)?;
        parsed_entries.push((path, entry.kind, entry.text));
    }

    tombi_fs::clear();
    for (path, kind, text) in parsed_entries {
        match kind {
            WorkspaceEntryKind::File => tombi_fs::set_file(path, text),
            WorkspaceEntryKind::Directory => tombi_fs::create_dir_all(&path)
                .map_err(|error| JsValue::from_str(&error.to_string()))?,
        }
    }
    Ok(())
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
