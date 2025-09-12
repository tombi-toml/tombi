use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, ErrorRuleOptions, TombiValueDirectiveContent, WithCommonLintRules,
    WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type IntegerFormatRules = EmptyFormatRules;

pub type KeyIntegerCommonLintRules = WithKeyTableLintRules<WithCommonLintRules<IntegerLintRules>>;

pub type IntegerCommonLintRules = WithCommonLintRules<IntegerLintRules>;

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<IntegerFormatRules, KeyIntegerCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-integer-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<IntegerFormatRules, IntegerCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-integer-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct IntegerLintRules {
    /// # Maximum integer
    ///
    /// Check if the integer is less than or equal to the maximum.
    ///
    /// ```rust
    /// integer <= maximum
    /// ```
    ///
    pub integer_maximum: Option<ErrorRuleOptions>,

    /// # Minimum integer
    ///
    /// Check if the integer is greater than or equal to the minimum.
    ///
    /// ```rust
    /// integer >= minimum
    /// ```
    ///
    pub integer_minimum: Option<ErrorRuleOptions>,

    /// # Exclusive maximum integer
    ///
    /// Check if the integer is less than the maximum.
    ///
    /// ```rust
    /// integer < maximum
    /// ```
    ///
    pub integer_exclusive_maximum: Option<ErrorRuleOptions>,

    /// # Exclusive minimum integer
    ///
    /// Check if the integer is greater than the minimum.
    ///
    /// ```rust
    /// integer > minimum
    /// ```
    ///
    pub integer_exclusive_minimum: Option<ErrorRuleOptions>,

    /// # Multiple of integer
    ///
    /// Check if the integer is a multiple of the value.
    ///
    /// ```rust
    /// integer % multiple_of == 0
    /// ```
    ///
    pub integer_multiple_of: Option<ErrorRuleOptions>,
}
