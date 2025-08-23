use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LocalDateKeyValueTombiCommentDirectiveRules {
    Key(KeyTombiCommentDirectiveRules),
    Value(LocalDateTombiCommentDirectiveRules),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LocalDateValueTombiCommentDirectiveRules {
    Common(CommonValueTombiCommentDirectiveRules),
    LocalDate(LocalDateTombiCommentDirectiveRules),
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LocalDateTombiCommentDirectiveRules {
    // No specific fields for local date type
}
