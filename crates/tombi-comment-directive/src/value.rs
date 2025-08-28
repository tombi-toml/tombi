mod array;
mod boolean;
mod float;
mod integer;
mod key;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;
mod table;

pub use array::*;
pub use boolean::*;
pub use float::*;
pub use integer::*;
pub use key::*;
pub use local_date::*;
pub use local_date_time::*;
pub use local_time::*;
pub use offset_date_time::*;
pub use string::*;
pub use table::*;
use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "Rules: serde::de::DeserializeOwned + serde::Serialize")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct ValueTombiCommentDirective<Rules> {
    pub lint: Option<ValueLintOptions<Rules>>,
}

impl<Rules> From<ValueTombiCommentDirective<WithKeyRules<Rules>>>
    for ValueTombiCommentDirective<Rules>
where
    Rules: From<WithKeyRules<Rules>> + serde::de::DeserializeOwned + serde::Serialize,
{
    fn from(value: ValueTombiCommentDirective<WithKeyRules<Rules>>) -> Self {
        Self {
            lint: value.lint.map(|lint| lint.into()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "Rules: serde::de::DeserializeOwned + serde::Serialize ")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct ValueLintOptions<Rules> {
    pub rules: Option<Rules>,
}

impl<Rules> From<ValueLintOptions<WithKeyRules<Rules>>> for ValueLintOptions<Rules>
where
    Rules: serde::de::DeserializeOwned + serde::Serialize + From<WithKeyRules<Rules>>,
{
    fn from(value: ValueLintOptions<WithKeyRules<Rules>>) -> Self {
        Self {
            rules: value.rules.map(|rules| rules.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithKeyRules<Rules> {
    #[serde(flatten)]
    pub key: KeyRules,

    #[serde(flatten)]
    pub value: Rules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithCommonRules<Rules> {
    #[serde(flatten)]
    pub common: CommonValueTombiCommentDirectiveRules,

    #[serde(flatten)]
    pub value: Rules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct CommonValueTombiCommentDirectiveRules {
    /// Controls the severity level for type mismatch errors
    pub type_mismatch: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for const value errors
    pub const_value: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for enumerate value errors
    pub enumerate: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for deprecated value errors
    pub deprecated: Option<SeverityLevelDefaultWarn>,
}
