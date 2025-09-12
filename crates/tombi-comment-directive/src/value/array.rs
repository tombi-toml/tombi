use std::str::FromStr;

use crate::value::{
    ErrorRuleOptions, TombiValueDirectiveContent, WithCommonRules, WithKeyTableRules,
};
use crate::TombiCommentDirectiveImpl;
use tombi_uri::SchemaUri;

pub type ArrayKeyCommonRules = WithKeyTableRules<WithCommonRules<ArrayRules>>;

pub type ArrayCommonRules = WithCommonRules<ArrayRules>;

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<ArrayKeyCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-array-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<ArrayCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-array-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ArrayRules {
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
