use tombi_severity_level::SeverityLevelDefaultError;

use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum StringKeyValueTombiCommentDirectiveRules {
    Key(KeyTombiCommentDirectiveRules),
    Value(StringTombiCommentDirectiveRules),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum StringValueTombiCommentDirectiveRules {
    Common(CommonValueTombiCommentDirectiveRules),
    String(StringTombiCommentDirectiveRules),
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct StringTombiCommentDirectiveRules {
    /// Controls the severity level for maximum length errors
    pub maximum_length: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum length errors
    pub minimum_length: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for format errors
    pub format: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for pattern errors
    pub pattern: Option<SeverityLevelDefaultError>,
}
