mod context;
mod document;
mod error;
mod value;

pub use context::CommentContext;
pub use error::Error;
use tombi_toml_version::TomlVersion;

pub const TOMBI_COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

pub use document::*;
use tombi_uri::SchemaUri;
pub use value::*;

pub trait TombiCommentDirectiveImpl {
    fn comment_directive_schema_url() -> SchemaUri;
}
