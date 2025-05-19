#[cfg(not(feature = "wasm"))]
pub fn url_from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<url::Url, ()> {
    url::Url::from_file_path(path)
}

#[cfg(feature = "wasm")]
pub fn url_from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<url::Url, ()> {
    Err(())
}

#[cfg(not(feature = "wasm"))]
pub fn url_to_file_path(url: &url::Url) -> Result<std::path::PathBuf, ()> {
    url.to_file_path()
}

#[cfg(feature = "wasm")]
pub fn url_to_file_path(url: &url::Url) -> Result<std::path::PathBuf, ()> {
    Err(())
}
