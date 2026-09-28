use serde::Deserialize;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

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

/// Replace the virtual workspace files visible to `format`/`lint`/`serve`.
#[wasm_bindgen]
pub fn set_workspace_files(files: JsValue) -> Result<(), JsValue> {
    set_workspace_entries(files)
}

/// Replace the virtual workspace entries visible to `format`/`lint`/`serve`.
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
