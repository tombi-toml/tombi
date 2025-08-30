use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type BooleanKeyValueTombiCommentDirective = ValueTombiCommentDirective<BooleanKeyValueRules>;

pub type BooleanValueTombiCommentDirective = ValueTombiCommentDirective<BooleanValueRules>;

pub type BooleanKeyValueRules = WithKeyRules<WithCommonRules<BooleanRules>>;

pub type BooleanValueRules = WithCommonRules<BooleanRules>;

impl TombiCommentDirectiveImpl for BooleanKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-boolean-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for BooleanValueTombiCommentDirective {
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
