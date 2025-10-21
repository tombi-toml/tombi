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

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "FormatRules: serde::de::DeserializeOwned, LintRules: serde::de::DeserializeOwned")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct TombiValueDirectiveContent<FormatRules, LintRules> {
    /// # Formatter options
    pub format: Option<FormatOptions<FormatRules>>,

    /// # Linter options
    pub lint: Option<LintOptions<LintRules>>,
}

impl<FormatRules, LintRules> TombiValueDirectiveContent<FormatRules, LintRules> {
    pub fn format_rules(&self) -> Option<&FormatRules> {
        if let TombiValueDirectiveContent {
            format: Some(FormatOptions { rules: Some(rules) }),
            ..
        } = self
        {
            Some(rules)
        } else {
            None
        }
    }

    pub fn lint_rules(&self) -> Option<&LintRules> {
        if let TombiValueDirectiveContent {
            lint: Some(LintOptions { rules: Some(rules) }),
            ..
        } = self
        {
            Some(rules)
        } else {
            None
        }
    }
}

impl<LintRules> TombiValueDirectiveContent<TableCommonFormatRules, LintRules> {
    #[inline]
    pub fn table_keys_order(&self) -> Option<SortMethod> {
        if let TombiValueDirectiveContent {
            format:
                Some(FormatOptions {
                    rules:
                        Some(TableCommonFormatRules {
                            value:
                                TableFormatRules {
                                    table_keys_order: Some(SortOptions::Sort(sort_method)),
                                    ..
                                },
                            ..
                        }),
                    ..
                }),
            ..
        } = self
        {
            Some(*sort_method)
        } else {
            None
        }
    }

    #[inline]
    pub fn table_keys_order_disabled(&self) -> Option<bool> {
        if let TombiValueDirectiveContent {
            format:
                Some(FormatOptions {
                    rules:
                        Some(TableCommonFormatRules {
                            value:
                                TableFormatRules {
                                    table_keys_order:
                                        Some(SortOptions::Disabled(SortDisabledOptions {
                                            disabled: Some(disabled),
                                        })),
                                    ..
                                },
                            ..
                        }),
                    ..
                }),
            ..
        } = self
        {
            Some(*disabled)
        } else {
            None
        }
    }
}

impl<LintRules> TombiValueDirectiveContent<ArrayCommonFormatRules, LintRules> {
    #[inline]
    pub fn array_values_order(&self) -> Option<SortMethod> {
        if let TombiValueDirectiveContent {
            format:
                Some(FormatOptions {
                    rules:
                        Some(WithCommonFormatRules {
                            value:
                                ArrayFormatRules {
                                    array_values_order: Some(SortOptions::Sort(sort_method)),
                                    ..
                                },
                            ..
                        }),
                    ..
                }),
            ..
        } = self
        {
            Some(*sort_method)
        } else {
            None
        }
    }

    #[inline]
    pub fn array_values_order_disabled(&self) -> Option<bool> {
        if let TombiValueDirectiveContent {
            format:
                Some(FormatOptions {
                    rules:
                        Some(WithCommonFormatRules {
                            value:
                                ArrayFormatRules {
                                    array_values_order:
                                        Some(SortOptions::Disabled(SortDisabledOptions {
                                            disabled: Some(disabled),
                                        })),
                                    ..
                                },
                            ..
                        }),
                    ..
                }),
            ..
        } = self
        {
            Some(*disabled)
        } else {
            None
        }
    }
}

impl<FormatRules, LintRules>
    From<TombiValueDirectiveContent<FormatRules, WithKeyTableLintRules<LintRules>>>
    for TombiValueDirectiveContent<FormatRules, LintRules>
where
    FormatRules: serde::de::DeserializeOwned,
    LintRules: From<WithKeyTableLintRules<LintRules>> + serde::de::DeserializeOwned,
{
    fn from(
        value: TombiValueDirectiveContent<FormatRules, WithKeyTableLintRules<LintRules>>,
    ) -> Self {
        Self {
            format: value.format,
            lint: value.lint.map(|lint| lint.into()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "FormatRules: serde::de::DeserializeOwned ")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct FormatOptions<FormatRules> {
    /// # Format rules
    pub rules: Option<FormatRules>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "LintRules: serde::de::DeserializeOwned  ")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct LintOptions<LintRules> {
    /// # Lint rules
    pub rules: Option<LintRules>,
}

impl<LintRules> From<LintOptions<WithKeyTableLintRules<LintRules>>> for LintOptions<LintRules>
where
    LintRules: serde::de::DeserializeOwned + From<WithKeyTableLintRules<LintRules>>,
{
    fn from(value: LintOptions<WithKeyTableLintRules<LintRules>>) -> Self {
        Self {
            rules: value.rules.map(|rules| rules.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct EmptyFormatRules {}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct CommonFormatRules {}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithCommonFormatRules<FormatRules> {
    #[serde(flatten)]
    pub common: CommonFormatRules,

    #[serde(flatten)]
    pub value: FormatRules,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithKeyFormatRules<FormatRules> {
    #[serde(flatten)]
    pub key: KeyFormatRules,

    #[serde(flatten)]
    pub value: FormatRules,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub enum SortOptions {
    Sort(SortMethod),
    Disabled(SortDisabledOptions),
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub enum SortMethod {
    Ascending,
    Descending,
    VersionSort,
}

impl From<SortMethod> for tombi_x_keyword::TableKeysOrder {
    fn from(val: SortMethod) -> Self {
        match val {
            SortMethod::Ascending => tombi_x_keyword::TableKeysOrder::Ascending,
            SortMethod::Descending => tombi_x_keyword::TableKeysOrder::Descending,
            SortMethod::VersionSort => tombi_x_keyword::TableKeysOrder::VersionSort,
        }
    }
}

impl From<SortMethod> for tombi_x_keyword::ArrayValuesOrder {
    fn from(val: SortMethod) -> Self {
        match val {
            SortMethod::Ascending => tombi_x_keyword::ArrayValuesOrder::Ascending,
            SortMethod::Descending => tombi_x_keyword::ArrayValuesOrder::Descending,
            SortMethod::VersionSort => tombi_x_keyword::ArrayValuesOrder::VersionSort,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct SortDisabledOptions {
    /// # Sort disabled
    ///
    /// If `true`, Sort is disabled for this value.
    ///
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_false"))]
    #[cfg_attr(feature = "jsonschema", schemars(extend("enum" = [true])))]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithKeyLintRules<LintRules> {
    #[serde(flatten)]
    pub key: KeyLinkRules,

    #[serde(flatten)]
    pub value: LintRules,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithCommonLintRules<LintRules> {
    #[serde(flatten)]
    pub common: CommonLintRules,

    #[serde(flatten)]
    pub value: LintRules,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithCommonExtensibleLintRules<LintRules> {
    #[serde(flatten)]
    pub common: CommonLintRules,

    #[serde(flatten)]
    pub value: LintRules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct WarnRuleOptions {
    /// # Warn rule disabled
    ///
    /// If `true`, Warn is disabled for this value.
    ///
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_false"))]
    #[cfg_attr(feature = "jsonschema", schemars(extend("enum" = [true])))]
    disabled: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct ErrorRuleOptions {
    /// # Error rule disabled
    ///
    /// If `true`, Error is disabled for this value.
    ///
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

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
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

    /// # One of multiple match
    ///
    /// Check if more than one schema in the `oneOf` is valid.
    ///
    pub one_of_multiple_match: Option<ErrorRuleOptions>,
}
