pub mod document;
mod error;
pub mod value;

pub use error::Error;
use tombi_toml_version::TomlVersion;

pub const TOMBI_COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

use tombi_uri::SchemaUri;

pub trait TombiCommentDirectiveImpl {
    fn comment_directive_schema_url() -> SchemaUri;
}

#[cfg(feature = "jsonschema")]
#[allow(unused)]
#[inline]
fn default_true() -> Option<bool> {
    Some(true)
}

#[cfg(feature = "jsonschema")]
#[allow(unused)]
#[inline]
fn default_false() -> Option<bool> {
    Some(false)
}
