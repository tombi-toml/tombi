use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirectiveContent, WithCommonRules, WithKeyTableRules};
use crate::TombiCommentDirectiveImpl;

pub type KeyLocalDateCommonRules = WithKeyTableRules<WithCommonRules<LocalDateRules>>;

pub type LocalDateCommonRules = WithCommonRules<LocalDateRules>;

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyLocalDateCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-date-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<LocalDateCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-local-date-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LocalDateRules {
    // No specific fields for local date type
}
