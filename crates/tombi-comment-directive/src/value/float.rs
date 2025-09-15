use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, ErrorRuleOptions, TombiValueDirectiveContent, WithCommonFormatRules,
    WithCommonLintRules, WithKeyFormatRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type FloatFormatRules = EmptyFormatRules;

pub type FloatCommonFormatRules = WithCommonFormatRules<FloatFormatRules>;
pub type FloatCommonLintRules = WithCommonLintRules<FloatLintRules>;

pub type KeyFloatCommonFormatRules = WithKeyFormatRules<FloatCommonFormatRules>;
pub type KeyFloatCommonLintRules = WithKeyTableLintRules<FloatCommonLintRules>;

pub type TombiFloatDirectiveContent =
    TombiValueDirectiveContent<FloatCommonFormatRules, FloatCommonLintRules>;

pub type TombiKeyFloatDirectiveContent =
    TombiValueDirectiveContent<KeyFloatCommonFormatRules, KeyFloatCommonLintRules>;

impl TombiCommentDirectiveImpl for TombiFloatDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-float-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyFloatDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-float-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct FloatLintRules {
    /// # Maximum float
    ///
    /// Check if the float is less than or equal to the maximum.
    ///
    /// ```rust
    /// float <= maximum
    /// ```
    ///
    pub float_maximum: Option<ErrorRuleOptions>,

    /// # Minimum float
    ///
    /// Check if the float is greater than or equal to the minimum.
    ///
    /// ```rust
    /// float >= minimum
    /// ```
    ///
    pub float_minimum: Option<ErrorRuleOptions>,

    /// # Exclusive maximum float
    ///
    /// Check if the float is less than the maximum.
    ///
    /// ```rust
    /// float < maximum
    /// ```
    ///
    pub float_exclusive_maximum: Option<ErrorRuleOptions>,

    /// # Exclusive minimum float
    ///
    /// Check if the float is greater than the minimum.
    ///
    /// ```rust
    /// float > minimum
    /// ```
    ///
    pub float_exclusive_minimum: Option<ErrorRuleOptions>,

    /// # Multiple of float
    ///
    /// Check if the float is a multiple of the value.
    ///
    /// ```rust
    /// float % multiple_of == 0
    /// ```
    ///
    pub float_multiple_of: Option<ErrorRuleOptions>,
}
