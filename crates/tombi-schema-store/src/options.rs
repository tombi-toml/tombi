#[derive(Debug, Clone, Default)]
pub struct Options {
    pub strict: Option<bool>,
    pub offline: Option<bool>,
    pub no_cache: Option<bool>,
    pub cache_ttl: Option<std::time::Duration>,
}

pub const DEFAULT_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);
