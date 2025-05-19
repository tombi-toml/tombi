#[cfg(not(feature = "wasm"))]
mod on_native;
#[cfg(not(feature = "wasm"))]
pub use on_native::*;

#[cfg(feature = "wasm")]
mod on_wasm;
#[cfg(feature = "wasm")]
pub use on_wasm::*;
