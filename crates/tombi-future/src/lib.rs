#[cfg(feature = "native")]
mod on_native;
#[cfg(feature = "native")]
pub use on_native::*;

#[cfg(not(feature = "native"))]
mod on_wasm;
#[cfg(not(feature = "native"))]
pub use on_wasm::*;
