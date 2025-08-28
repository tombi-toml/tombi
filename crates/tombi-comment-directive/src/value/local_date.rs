use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type LocalDateKeyValueTombiCommentDirective =
    ValueTombiCommentDirective<LocalDateKeyValueRules>;

pub type LocalDateValueTombiCommentDirective = ValueTombiCommentDirective<LocalDateValueRules>;

pub type LocalDateKeyValueRules = WithKeyRules<LocalDateRules>;

pub type LocalDateValueRules = WithCommonRules<LocalDateRules>;

impl TombiCommentDirectiveImpl for LocalDateKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/local-date-key-value-tombi-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for LocalDateValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/local-date-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LocalDateRules {
    // No specific fields for local date type
}
