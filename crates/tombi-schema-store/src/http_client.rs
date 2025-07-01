mod error;

#[cfg(all(feature = "wasm", feature = "surf"))]
compile_error!("Cannot enable both `wasm` and `surf` features");

#[cfg(not(any(feature = "wasm", feature = "surf")))]
mod reqwest_client;
#[cfg(not(any(feature = "wasm", feature = "surf")))]
pub use reqwest_client::HttpClient;

#[cfg(feature = "wasm")]
mod gloo_net_client;
#[cfg(feature = "wasm")]
pub use gloo_net_client::HttpClient;

#[cfg(feature = "surf")]
mod surf_client;
#[cfg(feature = "surf")]
pub use surf_client::HttpClient;
