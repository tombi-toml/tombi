use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonLintRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type OffsetDateTimeFormatRules = EmptyFormatRules;

pub type KeyOffsetDateTimeCommonLintRules =
    WithKeyTableLintRules<WithCommonLintRules<OffsetDateTimeLintRules>>;

pub type OffsetDateTimeCommonLintRules = WithCommonLintRules<OffsetDateTimeLintRules>;

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<OffsetDateTimeFormatRules, KeyOffsetDateTimeCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-offset-date-time-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<OffsetDateTimeFormatRules, OffsetDateTimeCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-offset-date-time-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct OffsetDateTimeLintRules {
    // No specific fields for offset date time type
}
