#[derive(Debug, Clone)]
pub struct Options {
    /// strict setting in global level.
    pub strict: Option<tombi_schema_type::BoolDefaultTrue>,
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
