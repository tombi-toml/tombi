use tombi_severity_level::SeverityLevelDefaultError;

use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct IntegerKeyValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    key: KeyTombiCommentDirectiveRules,

    #[serde(flatten)]
    value: IntegerTombiCommentDirectiveRules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct IntegerValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    common: CommonValueTombiCommentDirectiveRules,

    #[serde(flatten)]
    integer: IntegerTombiCommentDirectiveRules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct IntegerTombiCommentDirectiveRules {
    /// Controls the severity level for maximum integer errors
    pub integer_maximum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum integer errors
    pub integer_minimum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive maximum integer errors
    pub integer_exclusive_maximum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive minimum integer errors
    pub integer_exclusive_minimum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for multiple of integer errors
    pub integer_multiple_of: Option<SeverityLevelDefaultError>,
}
