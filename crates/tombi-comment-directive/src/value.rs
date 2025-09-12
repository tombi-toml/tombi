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
#[serde(bound = "LintRules: serde::de::DeserializeOwned + serde::Serialize")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct TombiValueDirectiveContent<LintRules> {
    /// # Linter options
    pub lint: Option<LintOptions<LintRules>>,
}

impl<LintRules> From<TombiValueDirectiveContent<WithKeyTableLintRules<LintRules>>>
    for TombiValueDirectiveContent<LintRules>
where
    LintRules:
        From<WithKeyTableLintRules<LintRules>> + serde::de::DeserializeOwned + serde::Serialize,
{
    fn from(value: TombiValueDirectiveContent<WithKeyTableLintRules<LintRules>>) -> Self {
        Self {
            lint: value.lint.map(|lint| lint.into()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "LintRules: serde::de::DeserializeOwned + serde::Serialize ")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LintOptions<LintRules> {
    /// # Lint rules
    pub rules: Option<LintRules>,
}

impl<LintRules> From<LintOptions<WithKeyTableLintRules<LintRules>>> for LintOptions<LintRules>
where
    LintRules:
        serde::de::DeserializeOwned + serde::Serialize + From<WithKeyTableLintRules<LintRules>>,
{
    fn from(value: LintOptions<WithKeyTableLintRules<LintRules>>) -> Self {
        Self {
            rules: value.rules.map(|rules| rules.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithKeyLintRules<LintRules> {
    #[serde(flatten)]
    pub key: KeyLinkRules,

    #[serde(flatten)]
    pub value: LintRules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithKeyTableLintRules<LintRules> {
    #[serde(flatten)]
    pub key: KeyLinkRules,

    #[serde(flatten)]
    pub table: TableLintRules,

    #[serde(flatten)]
    pub value: LintRules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithCommonLintRules<LintRules> {
    #[serde(flatten)]
    pub common: CommonLintRules,

    #[serde(flatten)]
    pub value: LintRules,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithCommonExtensibleLintRules<LintRules> {
    #[serde(flatten)]
    pub common: CommonLintRules,

    #[serde(flatten)]
    pub value: LintRules,
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
pub struct CommonLintRules {
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
