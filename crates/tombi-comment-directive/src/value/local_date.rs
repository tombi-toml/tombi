use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonFormatRules, WithCommonLintRules,
    WithKeyFormatRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type LocalDateFormatRules = EmptyFormatRules;

pub type LocalDateCommonFormatRules = WithCommonFormatRules<LocalDateFormatRules>;
pub type LocalDateCommonLintRules = WithCommonLintRules<LocalDateLintRules>;

pub type KeyLocalDateCommonFormatRules = WithKeyFormatRules<LocalDateCommonFormatRules>;
pub type KeyLocalDateCommonLintRules = WithKeyTableLintRules<LocalDateCommonLintRules>;

pub type TombiLocalDateDirectiveContent =
    TombiValueDirectiveContent<LocalDateCommonFormatRules, LocalDateCommonLintRules>;

pub type TombiKeyLocalDateDirectiveContent =
    TombiValueDirectiveContent<KeyLocalDateCommonFormatRules, KeyLocalDateCommonLintRules>;

impl TombiCommentDirectiveImpl for TombiLocalDateDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-local-date-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyLocalDateDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-date-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalDateLintRules {
    // No specific fields for local date type
}
