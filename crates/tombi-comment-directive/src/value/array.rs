use std::str::FromStr;

use crate::value::{TombiValueDirective, WithCommonRules, WithKeyRules};
use crate::TombiCommentDirectiveImpl;
use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

pub type TombiKeyArrayDirective = TombiValueDirective<ArrayKeyCommonRules>;

pub type TombiArrayDirective = TombiValueDirective<ArrayCommonRules>;

pub type ArrayKeyCommonRules = WithKeyRules<WithCommonRules<ArrayRules>>;

pub type ArrayCommonRules = WithCommonRules<ArrayRules>;

impl TombiCommentDirectiveImpl for TombiKeyArrayDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-array-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiArrayDirective {
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
    pub array_max_items: Option<SeverityLevelDefaultError>,

    /// # Minimum items.
    ///
    /// Check if the array has less than the minimum number of items.
    ///
    /// ```rust
    /// length(array) >= minimum
    /// ```
    ///
    pub array_min_items: Option<SeverityLevelDefaultError>,

    /// # Unique items.
    ///
    /// Check if the array has duplicate items.
    ///
    /// ```rust
    /// length(array) == length(unique(array))
    /// ```
    ///
    pub array_unique_items: Option<SeverityLevelDefaultError>,
}
