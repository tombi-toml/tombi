use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    EmptyFormatRules, ErrorRuleOptions, TombiValueDirectiveContent, WithCommonFormatRules,
    WithCommonLintRules, WithKeyFormatRules, WithKeyTableLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type StringFormatRules = EmptyFormatRules;

pub type StringCommonFormatRules = WithCommonFormatRules<StringFormatRules>;
pub type StringCommonLintRules = WithCommonLintRules<StringLintRules>;

pub type KeyStringCommonFormatRules = WithKeyFormatRules<StringCommonFormatRules>;
pub type KeyStringCommonLintRules = WithKeyTableLintRules<StringCommonLintRules>;

pub type TombiStringDirectiveContent =
    TombiValueDirectiveContent<StringCommonFormatRules, StringCommonLintRules>;

pub type TombiKeyStringDirectiveContent =
    TombiValueDirectiveContent<KeyStringCommonFormatRules, KeyStringCommonLintRules>;

impl TombiCommentDirectiveImpl for TombiStringDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-string-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyStringDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-string-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct StringLintRules {
    /// # Integer Max length
    ///
    /// Check if the string is longer than the max length.
    ///
    /// ```rust
    /// length(string) <= max
    /// ```
    ///
    pub string_max_length: Option<ErrorRuleOptions>,

    /// # Min length
    ///
    /// Check if the string is shorter than the min length.
    ///
    /// ```rust
    /// length(string) >= min
    /// ```
    ///
    pub string_min_length: Option<ErrorRuleOptions>,

    /// # String Format
    ///
    /// Check if the string matches the format.
    ///
    /// ```rust
    /// matches(string, format)
    /// ```
    ///
    pub string_format: Option<ErrorRuleOptions>,

    /// # String Pattern
    ///
    /// Check if the string matches the pattern.
    ///
    /// ```rust
    /// matches(string, pattern)
    /// ```
    ///
    pub string_pattern: Option<ErrorRuleOptions>,
}
