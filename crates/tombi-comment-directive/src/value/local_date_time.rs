use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirectiveContent, WithCommonRules, WithKeyTableRules};
use crate::TombiCommentDirectiveImpl;

pub type KeyLocalDateTimeCommonRules = WithKeyTableRules<WithCommonRules<LocalDateTimeRules>>;

pub type LocalDateTimeCommonRules = WithCommonRules<LocalDateTimeRules>;

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyLocalDateTimeCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-date-time-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<LocalDateTimeCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-local-date-time-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalDateTimeRules {
    // No specific fields for local date time type
}
