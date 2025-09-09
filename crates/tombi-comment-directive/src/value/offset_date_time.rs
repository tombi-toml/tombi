use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirectiveContent, WithCommonRules, WithKeyTableRules};
use crate::TombiCommentDirectiveImpl;

pub type KeyOffsetDateTimeCommonRules = WithKeyTableRules<WithCommonRules<OffsetDateTimeRules>>;

pub type OffsetDateTimeCommonRules = WithCommonRules<OffsetDateTimeRules>;

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyOffsetDateTimeCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-offset-date-time-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<OffsetDateTimeCommonRules> {
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
