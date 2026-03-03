#[derive(Debug, Clone)]
pub struct Options {
    pub strict: Option<bool>,
    pub dialect: Option<tombi_config::JsonSchemaDialect>,
    pub offline: Option<bool>,
    pub cache: Option<tombi_cache::Options>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            strict: None,
            dialect: None,
            offline: None,
            cache: Some(tombi_cache::Options::default()),
        }
    }
}
