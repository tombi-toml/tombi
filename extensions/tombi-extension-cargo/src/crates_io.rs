use serde::Deserialize;
use tombi_extension::fetch_cached_remote_json;
use tombi_hashmap::HashMap;

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

#[derive(Debug, Deserialize)]
pub(crate) struct CratesIoVersionsResponse {
    pub(crate) versions: Vec<CratesIoVersion>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CratesIoVersion {
    pub(crate) num: String,
    pub(crate) features: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CratesIoCrateVersionsResponse {
    #[serde(default)]
    pub(crate) versions: Vec<CratesIoVersion>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CratesIoVersionDetailResponse {
    pub(crate) version: CratesIoVersion,
}

pub(crate) async fn fetch_crates_io_crate(
    crate_name: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<CratesIoCrateResponse>, tower_lsp::jsonrpc::Error> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}");
    Ok(fetch_cached_remote_json::<CratesIoCrateResponse>(&url, offline, cache_options).await)
}
