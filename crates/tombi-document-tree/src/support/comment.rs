#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    String(#[from] tombi_toml_text::ParseError),
}
