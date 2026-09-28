/// Error returned by `format`/`lint`/[`crate::format_async`]/[`crate::lint_async`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Config(#[from] tombi_config::Error),

    #[error(transparent)]
    Schema(#[from] tombi_schema_store::Error),
}

impl Error {
    /// The error name shared by every binding: the exception class name in
    /// Python, and the `Error#name` of the rejected JS error in Node.js and
    /// wasm. This set is closed, so bindings can expose it as an exhaustive
    /// union type.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Io(_) => "TombiIOError",
            Self::Config(_) => "TombiConfigError",
            Self::Schema(_) => "TombiSchemaError",
        }
    }
}
