use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type LocalTimeKeyValueTombiCommentDirective =
    ValueTombiCommentDirective<LocalTimeKeyValueRules>;

pub type LocalTimeValueTombiCommentDirective = ValueTombiCommentDirective<LocalTimeValueRules>;

pub type LocalTimeKeyValueRules = WithKeyRules<LocalTimeRules>;

pub type LocalTimeValueRules = WithCommonRules<LocalTimeRules>;

impl TombiCommentDirectiveImpl for LocalTimeKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/local-time-key-value-tombi-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for LocalTimeValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/local-time-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LocalTimeRules {
    // No specific fields for local time type
}
