use std::str::FromStr;

use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type StringKeyValueTombiCommentDirective = ValueTombiCommentDirective<StringKeyValueRules>;

pub type StringValueTombiCommentDirective = ValueTombiCommentDirective<StringValueRules>;

pub type StringKeyValueRules = WithKeyRules<WithCommonRules<StringRules>>;

pub type StringValueRules = WithCommonRules<StringRules>;

impl TombiCommentDirectiveImpl for StringKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/string-key-value-tombi-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for StringValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/string-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct StringRules {
    /// Controls the severity level for maximum length errors
    pub string_max_length: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum length errors
    pub string_min_length: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for format errors
    pub string_format: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for pattern errors
    pub string_pattern: Option<SeverityLevelDefaultError>,
}
