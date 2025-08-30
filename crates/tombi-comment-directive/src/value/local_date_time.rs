use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirective, WithCommonRules, WithKeyRules};
use crate::TombiCommentDirectiveImpl;

pub type TombiKeyLocalDateTimeDirective = TombiValueDirective<KeyLocalDateTimeCommonRules>;

pub type TombiLocalDateTimeDirective = TombiValueDirective<LocalDateTimeCommonRules>;

pub type KeyLocalDateTimeCommonRules = WithKeyRules<WithCommonRules<LocalDateTimeRules>>;

pub type LocalDateTimeCommonRules = WithCommonRules<LocalDateTimeRules>;

impl TombiCommentDirectiveImpl for TombiKeyLocalDateTimeDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-date-time-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiLocalDateTimeDirective {
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
