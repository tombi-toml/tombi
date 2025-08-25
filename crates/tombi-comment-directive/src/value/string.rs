use std::str::FromStr;

use tombi_schema_store::SchemaUri;
use tombi_severity_level::SeverityLevelDefaultError;

use crate::{
    CommonValueTombiCommentDirectiveRules, ValueTombiCommentDirectiveImpl,
    WithKeyTombiCommentDirectiveRules,
};

impl ValueTombiCommentDirectiveImpl
    for WithKeyTombiCommentDirectiveRules<StringValueTombiCommentDirectiveRules>
{
    fn value_comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/string-key-value-tombi-directive.json").unwrap()
    }
}

impl From<WithKeyTombiCommentDirectiveRules<StringValueTombiCommentDirectiveRules>>
    for StringValueTombiCommentDirectiveRules
{
    fn from(
        rules: WithKeyTombiCommentDirectiveRules<StringValueTombiCommentDirectiveRules>,
    ) -> Self {
        rules.value
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct StringValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    common: CommonValueTombiCommentDirectiveRules,

    #[serde(flatten)]
    string: StringTombiCommentDirectiveRules,
}

impl ValueTombiCommentDirectiveImpl for StringValueTombiCommentDirectiveRules {
    fn value_comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/string-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct StringTombiCommentDirectiveRules {
    /// Controls the severity level for maximum length errors
    pub string_maximum_length: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum length errors
    pub string_minimum_length: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for format errors
    pub string_format: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for pattern errors
    pub string_pattern: Option<SeverityLevelDefaultError>,
}
