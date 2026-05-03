use serde::Deserialize;
use tombi_extension::fetch_cached_remote_json;

#[derive(Debug, Deserialize)]
pub(crate) struct CratesIoCrateResponse {
    #[serde(rename = "crate")]
    pub(crate) crate_info: CratesIoCrate,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CratesIoCrate {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) max_version: Option<String>,
}

pub(crate) async fn fetch_crates_io_crate(
    crate_name: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<CratesIoCrateResponse>, tower_lsp::jsonrpc::Error> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}");
    Ok(fetch_cached_remote_json::<CratesIoCrateResponse>(&url, offline, cache_options).await)
}

pub(crate) async fn fetch_latest_crates_io_version(
    crate_name: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<String>, tower_lsp::jsonrpc::Error> {
    let Some(response) = fetch_crates_io_crate(crate_name, offline, cache_options).await? else {
        return Ok(None);
    };

    Ok(response.crate_info.max_version)
}
