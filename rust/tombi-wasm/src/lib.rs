#[cfg(feature = "formatter")]
mod formatter;
#[cfg(feature = "lsp")]
mod lsp;

#[cfg(feature = "formatter")]
pub use formatter::{format, lint};
#[cfg(feature = "lsp")]
pub use lsp::{
    ServerConfig, remove_workspace_file, serve, set_workspace_entries, set_workspace_file,
    set_workspace_files,
};

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
