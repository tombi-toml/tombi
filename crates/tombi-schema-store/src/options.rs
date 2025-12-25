#[derive(Debug, Clone)]
pub struct Options {
    pub strict: Option<bool>,
    pub offline: Option<bool>,
    pub cache: Option<tombi_cache::Options>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            strict: None,
            offline: None,
            cache: Some(tombi_cache::Options::default()),
        }
    }
}
