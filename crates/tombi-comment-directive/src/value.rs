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
pub struct TombiValueDirectiveContent<Rules> {
    /// # Linter directive.
    pub lint: Option<LintOptions<Rules>>,
}

impl<Rules> From<TombiValueDirectiveContent<WithKeyRules<Rules>>>
    for TombiValueDirectiveContent<Rules>
where
    Rules: From<WithKeyRules<Rules>> + serde::de::DeserializeOwned + serde::Serialize,
{
    fn from(value: TombiValueDirectiveContent<WithKeyRules<Rules>>) -> Self {
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
pub struct LintOptions<Rules> {
    /// # Lint rules.
    pub rules: Option<Rules>,
}

impl<Rules> From<LintOptions<WithKeyRules<Rules>>> for LintOptions<Rules>
where
    Rules: serde::de::DeserializeOwned + serde::Serialize + From<WithKeyRules<Rules>>,
{
    fn from(value: LintOptions<WithKeyRules<Rules>>) -> Self {
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
    pub common: CommonRules,

    #[serde(flatten)]
    pub value: Rules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithCommonExtensibleRules<Rules> {
    #[serde(flatten)]
    pub common: CommonRules,

    #[serde(flatten)]
    pub value: Rules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct CommonRules {
    /// Type mismatch
    ///
    /// Check if the value is of the correct type.
    ///
    pub type_mismatch: Option<SeverityLevelDefaultError>,

    /// Const value
    ///
    /// Check if the value is equal to the const value.
    ///
    pub const_value: Option<SeverityLevelDefaultError>,

    /// Enumerate
    ///
    /// Check if the value is one of the values in the enumerate.
    ///
    pub enumerate: Option<SeverityLevelDefaultError>,

    /// Deprecated
    ///
    /// Check if the value is deprecated.
    ///
    pub deprecated: Option<SeverityLevelDefaultWarn>,
}
