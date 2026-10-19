#[derive(Debug, Clone)]
pub struct Options {
    /// strict setting in global level.
    pub strict: Option<tombi_schema_type::BoolDefaultTrue>,
    /// Global schema lint options from `[schema.lint]`.
    pub lint: Option<tombi_config::SchemaOverviewLintOptions>,
    /// Global schema format options from `[schema.format]`.
    pub format: Option<tombi_config::SchemaOverviewFormatOptions>,
    pub offline: Option<bool>,
    pub cache: Option<tombi_cache::Options>,
    /// Host names or IP addresses allowed to resolve to non-public addresses.
    ///
    /// Added to `TOMBI_SCHEMA_TRUSTED_HOSTS`. This is for the embedding
    /// application; project configuration cannot set it.
    pub trusted_hosts: Option<Vec<String>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            strict: None,
            lint: None,
            format: None,
            offline: None,
            cache: Some(tombi_cache::Options::default()),
            trusted_hosts: None,
        }
    }
}
