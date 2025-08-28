use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type BooleanKeyValueTombiCommentDirective = ValueTombiCommentDirective<BooleanKeyValueRules>;

pub type BooleanValueTombiCommentDirective = ValueTombiCommentDirective<BooleanValueRules>;

pub type BooleanKeyValueRules = WithKeyRules<BooleanRules>;

pub type BooleanValueRules = WithCommonRules<BooleanRules>;

impl TombiCommentDirectiveImpl for BooleanKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/boolean-key-value-tombi-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for BooleanValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/boolean-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct BooleanRules {
    // No specific fields for boolean type
}
