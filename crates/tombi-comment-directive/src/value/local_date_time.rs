use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LocalDateTimeKeyValueTombiCommentDirectiveRules {
    Key(KeyTombiCommentDirectiveRules),
    Value(LocalDateTimeTombiCommentDirectiveRules),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LocalDateTimeValueTombiCommentDirectiveRules {
    Common(CommonValueTombiCommentDirectiveRules),
    LocalDateTime(LocalDateTimeTombiCommentDirectiveRules),
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LocalDateTimeTombiCommentDirectiveRules {
    // No specific fields for local date time type
}
