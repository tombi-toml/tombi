use std::str::FromStr;

use crate::value::{
    ErrorRuleOptions, SortOptions, TombiValueDirectiveContent, WithCommonFormatRules,
    WithCommonLintRules, WithKeyFormatRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;
use tombi_uri::SchemaUri;

pub type ArrayCommonFormatRules = WithCommonFormatRules<ArrayFormatRules>;
pub type ArrayCommonLintRules = WithCommonLintRules<ArrayLintRules>;

pub type KeyArrayCommonFormatRules = WithKeyFormatRules<ArrayCommonFormatRules>;
pub type KeyArrayCommonLintRules = WithKeyTableLintRules<ArrayCommonLintRules>;

pub type TombiKeyArrayDirectiveContent =
    TombiValueDirectiveContent<KeyArrayCommonFormatRules, KeyArrayCommonLintRules>;

pub type TombiArrayDirectiveContent =
    TombiValueDirectiveContent<ArrayCommonFormatRules, ArrayCommonLintRules>;

impl TombiCommentDirectiveImpl for TombiArrayDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-array-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyArrayDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-array-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ArrayFormatRules {
    /// # Array values order
    ///
    /// Control the sorting method of the array.
    ///
    pub array_values_order: Option<SortOptions>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ArrayLintRules {
    /// # Max values
    ///
    /// Check if the array has more than the maximum number of values.
    ///
    /// ```rust
    /// length(array) <= maximum
    /// ```
    ///
    pub array_max_values: Option<ErrorRuleOptions>,

    /// # Min values
    ///
    /// Check if the array has less than the minimum number of values.
    ///
    /// ```rust
    /// length(array) >= minimum
    /// ```
    ///
    pub array_min_values: Option<ErrorRuleOptions>,

    /// # Unique values
    ///
    /// Check if the array has duplicate values.
    ///
    /// ```rust
    /// length(array) == length(unique(array))
    /// ```
    ///
    pub array_unique_values: Option<ErrorRuleOptions>,
}
