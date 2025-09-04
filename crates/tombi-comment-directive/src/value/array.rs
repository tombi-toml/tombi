use std::str::FromStr;

use crate::value::{ErrorRuleOptions, TombiValueDirectiveContent, WithCommonRules, WithKeyRules};
use crate::TombiCommentDirectiveImpl;
use tombi_uri::SchemaUri;

pub type TombiKeyArrayDirectiveContent = TombiValueDirectiveContent<ArrayKeyCommonRules>;

pub type TombiArrayDirectiveContent = TombiValueDirectiveContent<ArrayCommonRules>;

pub type ArrayKeyCommonRules = WithKeyRules<WithCommonRules<ArrayRules>>;

pub type ArrayCommonRules = WithCommonRules<ArrayRules>;

impl TombiCommentDirectiveImpl for TombiKeyArrayDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-array-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiArrayDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-array-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ArrayRules {
    /// # Maximum items.
    ///
    /// Check if the array has more than the maximum number of items.
    ///
    /// ```rust
    /// length(array) <= maximum
    /// ```
    ///
    pub array_max_items: Option<ErrorRuleOptions>,

    /// # Minimum items.
    ///
    /// Check if the array has less than the minimum number of items.
    ///
    /// ```rust
    /// length(array) >= minimum
    /// ```
    ///
    pub array_min_items: Option<ErrorRuleOptions>,

    /// # Unique items.
    ///
    /// Check if the array has duplicate items.
    ///
    /// ```rust
    /// length(array) == length(unique(array))
    /// ```
    ///
    pub array_unique_items: Option<ErrorRuleOptions>,
}
