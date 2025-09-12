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
use tombi_severity_level::{SeverityLevel, SeverityLevelDefaultError, SeverityLevelDefaultWarn};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "Rules: serde::de::DeserializeOwned + serde::Serialize")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct TombiValueDirectiveContent<Rules> {
    /// # Linter options
    pub lint: Option<LintOptions<Rules>>,
}

impl<Rules> From<TombiValueDirectiveContent<WithKeyTableRules<Rules>>>
    for TombiValueDirectiveContent<Rules>
where
    Rules: From<WithKeyTableRules<Rules>> + serde::de::DeserializeOwned + serde::Serialize,
{
    fn from(value: TombiValueDirectiveContent<WithKeyTableRules<Rules>>) -> Self {
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
    /// # Lint rules
    pub rules: Option<Rules>,
}

impl<Rules> From<LintOptions<WithKeyTableRules<Rules>>> for LintOptions<Rules>
where
    Rules: serde::de::DeserializeOwned + serde::Serialize + From<WithKeyTableRules<Rules>>,
{
    fn from(value: LintOptions<WithKeyTableRules<Rules>>) -> Self {
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
pub struct WithKeyTableRules<Rules> {
    #[serde(flatten)]
    pub key: KeyRules,

    #[serde(flatten)]
    pub table: TableRules,

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
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct WarnRuleOptions {
    /// # Warn rule disabled
    ///
    /// If `true`, Warn is disabled for this value.
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_false"))]
    #[cfg_attr(feature = "jsonschema", schemars(extend("enum" = [true])))]
    disabled: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ErrorRuleOptions {
    /// # Error rule disabled
    ///
    /// If `true`, Error is disabled for this value.
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_false"))]
    #[cfg_attr(feature = "jsonschema", schemars(extend("enum" = [true])))]
    disabled: Option<bool>,
}

impl From<&WarnRuleOptions> for SeverityLevelDefaultWarn {
    fn from(value: &WarnRuleOptions) -> Self {
        if value.disabled.unwrap_or(false) {
            SeverityLevel::Off.into()
        } else {
            Self::default()
        }
    }
}

impl From<&ErrorRuleOptions> for SeverityLevelDefaultError {
    fn from(value: &ErrorRuleOptions) -> Self {
        if value.disabled.unwrap_or(false) {
            SeverityLevel::Off.into()
        } else {
            Self::default()
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct CommonRules {
    /// # Type mismatch
    ///
    /// Check if the value is of the correct type.
    ///
    pub type_mismatch: Option<ErrorRuleOptions>,

    /// # Const value
    ///
    /// Check if the value is equal to the const value.
    ///
    pub const_value: Option<ErrorRuleOptions>,

    /// # Enumerate
    ///
    /// Check if the value is one of the values in the enumerate.
    ///
    pub enumerate: Option<ErrorRuleOptions>,

    /// # Deprecated
    ///
    /// Check if the value is deprecated.
    ///
    pub deprecated: Option<WarnRuleOptions>,
}
