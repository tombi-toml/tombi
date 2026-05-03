use serde::Deserialize;
use tombi_extension::fetch_cached_remote_json;

#[derive(Debug, Deserialize)]
pub(crate) struct PypiProjectResponse {
    pub(crate) info: PypiProjectInfo,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PypiProjectInfo {
    pub(crate) name: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) version: Option<String>,
}

pub(crate) async fn fetch_pypi_project(
    package_name: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<PypiProjectResponse>, tower_lsp::jsonrpc::Error> {
    let url = format!("https://pypi.org/pypi/{package_name}/json");
    Ok(fetch_cached_remote_json::<PypiProjectResponse>(&url, offline, cache_options).await)
}
