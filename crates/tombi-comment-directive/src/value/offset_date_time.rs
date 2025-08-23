use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct OffsetDateTimeKeyValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    key: KeyTombiCommentDirectiveRules,

    #[serde(flatten)]
    value: OffsetDateTimeTombiCommentDirectiveRules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct OffsetDateTimeValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    common: CommonValueTombiCommentDirectiveRules,

    #[serde(flatten)]
    offset_date_time: OffsetDateTimeTombiCommentDirectiveRules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct OffsetDateTimeTombiCommentDirectiveRules {
    // No specific fields for offset date time type
}
