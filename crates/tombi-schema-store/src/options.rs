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
}

impl Default for Options {
    fn default() -> Self {
        Self {
            strict: None,
            lint: None,
            format: None,
            offline: None,
            cache: Some(tombi_cache::Options::default()),
        }
    }
}
