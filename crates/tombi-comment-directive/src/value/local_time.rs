use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonLintRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type LocalTimeFormatRules = EmptyFormatRules;

pub type KeyLocalTimeCommonLintRules =
    WithKeyTableLintRules<WithCommonLintRules<LocalTimeLintRules>>;

pub type LocalTimeCommonLintRules = WithCommonLintRules<LocalTimeLintRules>;

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<LocalTimeFormatRules, KeyLocalTimeCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-time-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<LocalTimeFormatRules, LocalTimeCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-local-time-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalTimeLintRules {
    // No specific fields for local time type
}
