use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonLintRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type BooleanFormatRules = EmptyFormatRules;

pub type KeyBooleanCommonLintRules = WithKeyTableLintRules<WithCommonLintRules<BooleanLintRules>>;

pub type BooleanCommonLintRules = WithCommonLintRules<BooleanLintRules>;

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<BooleanFormatRules, KeyBooleanCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-boolean-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<BooleanFormatRules, BooleanCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-boolean-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct BooleanLintRules {
    // No specific fields for boolean type
}
