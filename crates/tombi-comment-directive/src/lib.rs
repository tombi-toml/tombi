mod context;
pub mod document;
mod error;
pub mod value;

pub use error::Error;
use tombi_toml_version::TomlVersion;

pub const TOMBI_COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

pub use context::{CommentDirectiveContext, GetCommentDirectiveContext};
use tombi_uri::SchemaUri;

use crate::value::{ArrayCommonRules, TombiValueDirectiveContent, WithKeyRules};

pub trait TombiCommentDirectiveImpl {
    fn comment_directive_schema_url() -> SchemaUri;
}

pub fn get_value_comment_directive_content_with_schema_uri<Rules>(
    comment_directives: Option<&[tombi_ast::TombiValueCommentDirective]>,
    position: tombi_text::Position,
    accessors: &[tombi_accessor::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)>
where
    TombiValueDirectiveContent<Rules>: TombiCommentDirectiveImpl,
    TombiValueDirectiveContent<WithKeyRules<Rules>>: TombiCommentDirectiveImpl,
{
    if let Some(comment_directive) = comment_directives {
        for comment_directive in comment_directive {
            if let Some(comment_directive_context) = comment_directive.get_context(position) {
                let schema_uri = if let Some(tombi_accessor::Accessor::Index(_)) = accessors.last()
                {
                    TombiValueDirectiveContent::<Rules>::comment_directive_schema_url()
                } else {
                    TombiValueDirectiveContent::<WithKeyRules<Rules>>::comment_directive_schema_url(
                    )
                };
                return Some((comment_directive_context, schema_uri));
            }
        }
    }
    None
}

pub fn get_array_comment_directive_content_with_schema_uri(
    array: &tombi_document_tree::Array,
    position: tombi_text::Position,
    accessors: &[tombi_accessor::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)> {
    if let Some((comment_directive, schema_uri)) =
        get_value_comment_directive_content_with_schema_uri::<ArrayCommonRules>(
            array.comment_directives(),
            position,
            accessors,
        )
    {
        return Some((comment_directive, schema_uri));
    }
    if let Some(comment_directive) = array.inner_comment_directives() {
        for comment_directive in comment_directive {
            if let Some(comment_directive_context) = comment_directive.get_context(position) {
                let schema_uri =
                    TombiValueDirectiveContent::<ArrayCommonRules>::comment_directive_schema_url();
                return Some((comment_directive_context, schema_uri));
            }
        }
    }

    None
}
