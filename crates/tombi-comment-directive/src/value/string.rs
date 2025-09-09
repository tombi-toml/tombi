use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    ErrorRuleOptions, TombiValueDirectiveContent, WithCommonRules, WithKeyTableRules,
};
use crate::TombiCommentDirectiveImpl;

pub type KeyStringCommonRules = WithKeyTableRules<WithCommonRules<StringRules>>;

pub type StringCommonRules = WithCommonRules<StringRules>;

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyStringCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-string-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<StringCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-string-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct StringRules {
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
