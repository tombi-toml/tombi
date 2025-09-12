use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirectiveContent, WithCommonLintRules, WithKeyTableLintRules};
use crate::TombiCommentDirectiveImpl;

pub type KeyLocalDateCommonLintRules =
    WithKeyTableLintRules<WithCommonLintRules<LocalDateLintRules>>;

pub type LocalDateCommonLintRules = WithCommonLintRules<LocalDateLintRules>;

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyLocalDateCommonLintRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-date-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<LocalDateCommonLintRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-local-date-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalDateLintRules {
    // No specific fields for local date type
}
