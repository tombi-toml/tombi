use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type LocalTimeKeyValueTombiCommentDirective =
    ValueTombiCommentDirective<LocalTimeKeyValueRules>;

pub type LocalTimeValueTombiCommentDirective = ValueTombiCommentDirective<LocalTimeValueRules>;

pub type LocalTimeKeyValueRules = WithKeyRules<WithCommonRules<LocalTimeRules>>;

pub type LocalTimeValueRules = WithCommonRules<LocalTimeRules>;

impl TombiCommentDirectiveImpl for LocalTimeKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-local-time-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for LocalTimeValueTombiCommentDirective {
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
