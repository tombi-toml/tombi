use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type LocalDateTimeKeyValueTombiCommentDirective =
    ValueTombiCommentDirective<LocalDateTimeKeyValueRules>;

pub type LocalDateTimeValueTombiCommentDirective =
    ValueTombiCommentDirective<LocalDateTimeValueRules>;

pub type LocalDateTimeKeyValueRules = WithKeyRules<WithCommonRules<LocalDateTimeRules>>;

pub type LocalDateTimeValueRules = WithCommonRules<LocalDateTimeRules>;

impl TombiCommentDirectiveImpl for LocalDateTimeKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/local-date-time-key-value-tombi-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for LocalDateTimeValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/local-date-time-value-tombi-directive.json")
            .unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LocalDateTimeRules {
    // No specific fields for local date time type
}
