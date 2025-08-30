use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirectiveContent, WithCommonRules, WithKeyRules};
use crate::TombiCommentDirectiveImpl;

pub type TombiKeyOffsetDateTimeDirectiveContent =
    TombiValueDirectiveContent<KeyOffsetDateTimeCommonRules>;

pub type TombiOffsetDateTimeDirectiveContent =
    TombiValueDirectiveContent<OffsetDateTimeCommonRules>;

pub type KeyOffsetDateTimeCommonRules = WithKeyRules<WithCommonRules<OffsetDateTimeRules>>;

pub type OffsetDateTimeCommonRules = WithCommonRules<OffsetDateTimeRules>;

impl TombiCommentDirectiveImpl for TombiKeyOffsetDateTimeDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-offset-date-time-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiOffsetDateTimeDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-offset-date-time-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct OffsetDateTimeRules {
    // No specific fields for offset date time type
}
