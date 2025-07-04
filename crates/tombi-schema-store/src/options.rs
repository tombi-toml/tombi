#[derive(Debug, Clone, Default)]
pub struct Options {
    pub strict: Option<bool>,
    pub offline: Option<bool>,
    pub cache: Option<tombi_cache::Options>,
}
