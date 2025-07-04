mod error;

#[cfg(feature = "reqwest01")]
mod reqwest_client;
#[cfg(feature = "reqwest01")]
pub use reqwest_client::HttpClient;

#[cfg(feature = "gloo-net06")]
#[allow(dead_code)]
mod gloo_net_client;
#[cfg(all(feature = "gloo-net06", not(feature = "wasm")))]
pub use gloo_net_client::HttpClient;

#[cfg(feature = "surf2")]
mod surf_client;
#[cfg(feature = "surf2")]
pub use surf_client::HttpClient;
