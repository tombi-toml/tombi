#[cfg(feature = "lib")]
mod formatter;
#[cfg(feature = "lsp")]
mod lsp;
#[cfg(any(feature = "lib", feature = "lsp"))]
mod workspace;

#[cfg(feature = "lib")]
pub use formatter::{format, lint};
#[cfg(feature = "lsp")]
pub use lsp::{ServerConfig, serve};
#[cfg(any(feature = "lib", feature = "lsp"))]
pub use workspace::{
    remove_workspace_file, set_workspace_entries, set_workspace_file, set_workspace_files,
};

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
