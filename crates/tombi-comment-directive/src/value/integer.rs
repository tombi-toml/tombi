use std::str::FromStr;

use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type IntegerKeyValueTombiCommentDirective = ValueTombiCommentDirective<IntegerKeyValueRules>;

pub type IntegerValueTombiCommentDirective = ValueTombiCommentDirective<IntegerValueRules>;

pub type IntegerKeyValueRules = WithKeyRules<WithCommonRules<IntegerRules>>;

pub type IntegerValueRules = WithCommonRules<IntegerRules>;

impl TombiCommentDirectiveImpl for IntegerKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-integer-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for IntegerValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-integer-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct IntegerRules {
    /// # Maximum integer.
    ///
    /// Check if the integer is less than or equal to the maximum.
    ///
    /// ```rust
    /// integer <= maximum
    /// ```
    ///
    pub integer_maximum: Option<SeverityLevelDefaultError>,

    /// # Minimum integer.
    ///
    /// Check if the integer is greater than or equal to the minimum.
    ///
    /// ```rust
    /// integer >= minimum
    /// ```
    ///
    pub integer_minimum: Option<SeverityLevelDefaultError>,

    /// # Exclusive maximum integer.
    ///
    /// Check if the integer is less than the maximum.
    ///
    /// ```rust
    /// integer < maximum
    /// ```
    ///
    pub integer_exclusive_maximum: Option<SeverityLevelDefaultError>,

    /// # Exclusive minimum integer.
    ///
    /// Check if the integer is greater than the minimum.
    ///
    /// ```rust
    /// integer > minimum
    /// ```
    ///
    pub integer_exclusive_minimum: Option<SeverityLevelDefaultError>,

    /// # Multiple of integer.
    ///
    /// Check if the integer is a multiple of the value.
    ///
    /// ```rust
    /// integer % multiple_of == 0
    /// ```
    ///
    pub integer_multiple_of: Option<SeverityLevelDefaultError>,
}
