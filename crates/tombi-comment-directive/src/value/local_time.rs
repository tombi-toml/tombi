use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::TombiCommentDirectiveImpl;
use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonFormatRules, WithCommonLintRules,
    WithKeyFormatRules, WithKeyTableLintRules,
};

pub type LocalTimeFormatRules = EmptyFormatRules;

pub type LocalTimeCommonFormatRules = WithCommonFormatRules<LocalTimeFormatRules>;
pub type LocalTimeCommonLintRules = WithCommonLintRules<LocalTimeLintRules>;

pub type KeyLocalTimeCommonFormatRules = WithKeyFormatRules<LocalTimeCommonFormatRules>;
pub type KeyLocalTimeCommonLintRules = WithKeyTableLintRules<LocalTimeCommonLintRules>;

pub type TombiLocalTimeDirectiveContent =
    TombiValueDirectiveContent<LocalTimeCommonFormatRules, LocalTimeCommonLintRules>;

pub type TombiKeyLocalTimeDirectiveContent =
    TombiValueDirectiveContent<KeyLocalTimeCommonFormatRules, KeyLocalTimeCommonLintRules>;

impl TombiCommentDirectiveImpl for TombiLocalTimeDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://www.schemastore.tombi/tombi-local-time-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyLocalTimeDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://www.schemastore.tombi/tombi-key-local-time-directive.json")
            .unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalTimeLintRules {
    // No specific fields for local time type
}
