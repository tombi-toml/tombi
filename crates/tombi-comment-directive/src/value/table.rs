use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};

use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct TableKeyValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    key: KeyTombiCommentDirectiveRules,

    #[serde(flatten)]
    value: TableTombiCommentDirectiveRules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct TableValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    common: CommonValueTombiCommentDirectiveRules,

    #[serde(flatten)]
    table: TableTombiCommentDirectiveRules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct TableTombiCommentDirectiveRules {
    /// # Tables out of order.
    ///
    /// Check if tables are defined out of order.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// [fruit.apple]
    /// [animal]
    /// [fruit.orange]
    ///
    /// # RECOMMENDED
    /// [fruit.apple]
    /// [fruit.orange]
    /// [animal]
    /// ```
    pub tables_out_of_order: Option<SeverityLevelDefaultWarn>,

    /// Controls the severity level for max properties errors
    pub table_max_properties: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for min properties errors
    pub table_min_properties: Option<SeverityLevelDefaultError>,
}
