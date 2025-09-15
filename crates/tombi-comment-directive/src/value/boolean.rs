use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, TombiValueDirectiveContent, WithCommonFormatRules, WithCommonLintRules,
    WithKeyFormatRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type BooleanFormatRules = EmptyFormatRules;

pub type BooleanCommonFormatRules = WithCommonFormatRules<BooleanFormatRules>;
pub type BooleanCommonLintRules = WithCommonLintRules<BooleanLintRules>;

pub type KeyBooleanCommonFormatRules = WithKeyFormatRules<BooleanCommonFormatRules>;
pub type KeyBooleanCommonLintRules = WithKeyTableLintRules<BooleanCommonLintRules>;

pub type TombiBooleanDirectiveContent =
    TombiValueDirectiveContent<BooleanCommonFormatRules, BooleanCommonLintRules>;

pub type TombiKeyBooleanDirectiveContent =
    TombiValueDirectiveContent<KeyBooleanCommonFormatRules, KeyBooleanCommonLintRules>;

impl TombiCommentDirectiveImpl for TombiBooleanDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-boolean-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyBooleanDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-boolean-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct BooleanLintRules {
    // No specific fields for boolean type
}
