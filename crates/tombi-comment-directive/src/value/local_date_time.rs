use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonFormatRules, WithCommonLintRules,
    WithKeyFormatRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type LocalDateTimeFormatRules = EmptyFormatRules;

pub type LocalDateTimeCommonFormatRules = WithCommonFormatRules<LocalDateTimeFormatRules>;
pub type LocalDateTimeCommonLintRules = WithCommonLintRules<LocalDateTimeLintRules>;

pub type KeyLocalDateTimeCommonFormatRules = WithKeyFormatRules<LocalDateTimeCommonFormatRules>;
pub type KeyLocalDateTimeCommonLintRules = WithKeyTableLintRules<LocalDateTimeCommonLintRules>;

pub type TombiLocalDateTimeDirectiveContent =
    TombiValueDirectiveContent<LocalDateTimeCommonFormatRules, LocalDateTimeCommonLintRules>;

pub type TombiKeyLocalDateTimeDirectiveContent =
    TombiValueDirectiveContent<KeyLocalDateTimeCommonFormatRules, KeyLocalDateTimeCommonLintRules>;

impl TombiCommentDirectiveImpl for TombiLocalDateTimeDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-local-date-time-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyLocalDateTimeDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-date-time-directive.json")
            .unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalDateTimeLintRules {
    // No specific fields for local date time type
}
