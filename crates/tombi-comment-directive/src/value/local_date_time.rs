use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonLintRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type LocalDateTimeFormatRules = EmptyFormatRules;

pub type KeyLocalDateTimeCommonLintRules =
    WithKeyTableLintRules<WithCommonLintRules<LocalDateTimeLintRules>>;

pub type LocalDateTimeCommonLintRules = WithCommonLintRules<LocalDateTimeLintRules>;

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<LocalDateTimeFormatRules, KeyLocalDateTimeCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-date-time-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<LocalDateTimeFormatRules, LocalDateTimeCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-local-date-time-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalDateTimeLintRules {
    // No specific fields for local date time type
}
