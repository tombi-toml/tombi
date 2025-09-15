use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonFormatRules, WithCommonLintRules,
    WithKeyFormatRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type OffsetDateTimeFormatRules = EmptyFormatRules;

pub type OffsetDateTimeCommonFormatRules = WithCommonFormatRules<OffsetDateTimeFormatRules>;
pub type OffsetDateTimeCommonLintRules = WithCommonLintRules<OffsetDateTimeLintRules>;

pub type KeyOffsetDateTimeCommonFormatRules = WithKeyFormatRules<OffsetDateTimeCommonFormatRules>;
pub type KeyOffsetDateTimeCommonLintRules = WithKeyTableLintRules<OffsetDateTimeCommonLintRules>;

pub type TombiOffsetDateTimeDirectiveContent =
    TombiValueDirectiveContent<OffsetDateTimeCommonFormatRules, OffsetDateTimeCommonLintRules>;

pub type TombiKeyOffsetDateTimeDirectiveContent = TombiValueDirectiveContent<
    KeyOffsetDateTimeCommonFormatRules,
    KeyOffsetDateTimeCommonLintRules,
>;

impl TombiCommentDirectiveImpl for TombiOffsetDateTimeDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-offset-date-time-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyOffsetDateTimeDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-offset-date-time-directive.json")
            .unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct OffsetDateTimeLintRules {
    // No specific fields for offset date time type
}
