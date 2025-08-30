use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirective, WithCommonRules, WithKeyRules};
use crate::TombiCommentDirectiveImpl;

pub type TombiKeyLocalTimeDirective = TombiValueDirective<KeyLocalTimeCommonRules>;

pub type TombiLocalTimeDirective = TombiValueDirective<LocalTimeCommonRules>;

pub type KeyLocalTimeCommonRules = WithKeyRules<WithCommonRules<LocalTimeRules>>;

pub type LocalTimeCommonRules = WithCommonRules<LocalTimeRules>;

impl TombiCommentDirectiveImpl for TombiKeyLocalTimeDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-time-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiLocalTimeDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-local-time-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalTimeRules {
    // No specific fields for local time type
}
