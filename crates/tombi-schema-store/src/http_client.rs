mod error;

#[cfg(feature = "reqwest01")]
mod reqwest_client;
#[cfg(feature = "reqwest01")]
pub use reqwest_client::HttpClient;

#[cfg(feature = "gloo-net06")]
mod gloo_net_client;
#[cfg(feature = "gloo-net06")]
pub use gloo_net_client::HttpClient;

#[cfg(feature = "surf2")]
mod surf_client;
#[cfg(feature = "surf2")]
pub use surf_client::HttpClient;
