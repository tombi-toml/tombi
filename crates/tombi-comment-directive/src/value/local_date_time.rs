use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct LocalDateTimeKeyValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    key: KeyTombiCommentDirectiveRules,

    #[serde(flatten)]
    value: LocalDateTimeTombiCommentDirectiveRules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LocalDateTimeValueTombiCommentDirectiveRules {
    #[serde(flatten)]
    common: CommonValueTombiCommentDirectiveRules,

    #[serde(flatten)]
    local_date_time: LocalDateTimeTombiCommentDirectiveRules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LocalDateTimeTombiCommentDirectiveRules {
    // No specific fields for local date time type
}
