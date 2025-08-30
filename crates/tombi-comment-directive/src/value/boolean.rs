use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirectiveContent, WithCommonRules, WithKeyRules};
use crate::TombiCommentDirectiveImpl;

pub type TombiKeyBooleanDirectiveContent = TombiValueDirectiveContent<KeyBooleanCommonRules>;

pub type TombiBooleanDirectiveContent = TombiValueDirectiveContent<BooleanCommonRules>;

pub type KeyBooleanCommonRules = WithKeyRules<WithCommonRules<BooleanRules>>;

pub type BooleanCommonRules = WithCommonRules<BooleanRules>;

impl TombiCommentDirectiveImpl for TombiKeyBooleanDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-boolean-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiBooleanDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-boolean-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct BooleanRules {
    // No specific fields for boolean type
}
