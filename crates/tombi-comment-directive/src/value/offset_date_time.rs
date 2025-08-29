use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type OffsetDateTimeKeyValueTombiCommentDirective =
    ValueTombiCommentDirective<OffsetDateTimeKeyValueRules>;

pub type OffsetDateTimeValueTombiCommentDirective =
    ValueTombiCommentDirective<OffsetDateTimeValueRules>;

pub type OffsetDateTimeKeyValueRules = WithKeyRules<WithCommonRules<OffsetDateTimeRules>>;

pub type OffsetDateTimeValueRules = WithCommonRules<OffsetDateTimeRules>;

impl TombiCommentDirectiveImpl for OffsetDateTimeKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str(
            "tombi://json.tombi.dev/offset-date-time-key-value-tombi-directive.json",
        )
        .unwrap()
    }
}

impl TombiCommentDirectiveImpl for OffsetDateTimeValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/offset-date-time-value-tombi-directive.json")
            .unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct OffsetDateTimeRules {
    // No specific fields for offset date time type
}
