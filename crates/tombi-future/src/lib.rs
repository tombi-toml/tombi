#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod on_native;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use on_native::*;

#[cfg(any(not(feature = "native"), target_arch = "wasm32"))]
mod on_wasm;
#[cfg(any(not(feature = "native"), target_arch = "wasm32"))]
pub use on_wasm::*;
