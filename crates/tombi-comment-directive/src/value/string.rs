use std::str::FromStr;

use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type StringKeyValueTombiCommentDirective = ValueTombiCommentDirective<StringKeyValueRules>;

pub type StringValueTombiCommentDirective = ValueTombiCommentDirective<StringValueRules>;

pub type StringKeyValueRules = WithKeyRules<WithCommonRules<StringRules>>;

pub type StringValueRules = WithCommonRules<StringRules>;

impl TombiCommentDirectiveImpl for StringKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-string-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for StringValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-string-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct StringRules {
    /// # Maximum length.
    ///
    /// Check if the string is longer than the maximum length.
    ///
    /// ```rust
    /// length(string) <= maximum
    /// ```
    ///
    pub string_max_length: Option<SeverityLevelDefaultError>,

    /// # Minimum length.
    ///
    /// Check if the string is shorter than the minimum length.
    ///
    /// ```rust
    /// length(string) >= minimum
    /// ```
    ///
    pub string_min_length: Option<SeverityLevelDefaultError>,

    /// # Format.
    ///
    /// Check if the string matches the format.
    ///
    /// ```rust
    /// matches(string, format)
    /// ```
    ///
    pub string_format: Option<SeverityLevelDefaultError>,

    /// # Pattern.
    ///
    /// Check if the string matches the pattern.
    ///
    /// ```rust
    /// matches(string, pattern)
    /// ```
    ///
    pub string_pattern: Option<SeverityLevelDefaultError>,
}
