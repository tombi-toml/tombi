use crate::{CommonValueTombiCommentDirectiveRules, KeyTombiCommentDirectiveRules};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum BooleanKeyValueTombiCommentDirectiveRules {
    Key(KeyTombiCommentDirectiveRules),
    Value(BooleanTombiCommentDirectiveRules),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum BooleanValueTombiCommentDirectiveRules {
    Common(CommonValueTombiCommentDirectiveRules),
    Boolean(BooleanTombiCommentDirectiveRules),
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct BooleanTombiCommentDirectiveRules {
    // No specific fields for boolean type
}
