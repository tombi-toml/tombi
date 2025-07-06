#[cfg(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
))]
#[allow(clippy::result_unit_err)]
pub fn url_from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<url::Url, ()> {
    url::Url::from_file_path(path)
}

#[cfg(not(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
)))]
#[allow(clippy::result_unit_err)]
pub fn url_from_file_path<P: AsRef<std::path::Path>>(_path: P) -> Result<url::Url, ()> {
    Err(())
}

#[cfg(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
))]
#[allow(clippy::result_unit_err)]
pub fn url_to_file_path(url: &url::Url) -> Result<std::path::PathBuf, ()> {
    url.to_file_path()
}

#[cfg(not(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
)))]
#[allow(clippy::result_unit_err)]
pub fn url_to_file_path(_url: &url::Url) -> Result<std::path::PathBuf, ()> {
    Err(())
}
