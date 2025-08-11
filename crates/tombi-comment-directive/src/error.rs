#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Invalid TOML version: {0}")]
    InvalidTomlVersion(String),
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub range: Option<tombi_text::Range>,
}
