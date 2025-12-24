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
                .map(|value| std::time::Duration::from_secs(value))
                .or_else(|| Some(DEFAULT_CACHE_TTL)),
        }
    }
}

pub const DEFAULT_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);
