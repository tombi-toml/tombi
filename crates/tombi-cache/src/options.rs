#[derive(Debug, Clone)]
pub struct Options {
    pub no_cache: Option<bool>,
    pub cache_ttl: Option<std::time::Duration>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            no_cache: None,
            cache_ttl: std::env::var("TOMBI_CACHE_TTL")
                .map_or(None, |value| value.parse::<u64>().ok())
                .map(std::time::Duration::from_secs)
                .or(Some(DEFAULT_CACHE_TTL_DURATION)),
        }
    }
}

pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 60 * 60 * 24;
pub const DEFAULT_CACHE_TTL_DURATION: std::time::Duration =
    std::time::Duration::from_secs(DEFAULT_CACHE_TTL_SECONDS);
