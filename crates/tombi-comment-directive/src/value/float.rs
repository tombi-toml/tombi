use tombi_severity_level::SeverityLevelDefaultError;

use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct FloatKeyValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    key: KeyTombiCommentDirectiveRules,

    #[serde(flatten)]
    value: FloatTombiCommentDirectiveRules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct FloatValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    common: CommonValueTombiCommentDirectiveRules,

    #[serde(flatten)]
    float: FloatTombiCommentDirectiveRules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct FloatTombiCommentDirectiveRules {
    /// Controls the severity level for maximum float errors
    pub maximum_float: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum float errors
    pub minimum_float: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive maximum float errors
    pub exclusive_maximum_float: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive minimum float errors
    pub exclusive_minimum_float: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for multiple of float errors
    pub multiple_of_float: Option<SeverityLevelDefaultError>,
}
