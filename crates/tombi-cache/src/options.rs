#[derive(Debug, Clone)]
pub struct Options {
    pub no_cache: Option<bool>,
    pub cache_ttl: Option<std::time::Duration>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            no_cache: None,
            cache_ttl: Some(DEFAULT_CACHE_TTL),
        }
    }
}

pub const DEFAULT_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);
