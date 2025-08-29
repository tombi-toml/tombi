use std::str::FromStr;

use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type FloatKeyValueTombiCommentDirective = ValueTombiCommentDirective<FloatKeyValueRules>;

pub type FloatValueTombiCommentDirective = ValueTombiCommentDirective<FloatValueRules>;

pub type FloatKeyValueRules = WithKeyRules<WithCommonRules<FloatRules>>;

pub type FloatValueRules = WithCommonRules<FloatRules>;

impl TombiCommentDirectiveImpl for FloatKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/float-key-value-tombi-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for FloatValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/float-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct FloatRules {
    /// # Maximum float.
    ///
    /// Check if the float is less than or equal to the maximum.
    ///
    /// ```rust
    /// float <= maximum
    /// ```
    ///
    pub float_maximum: Option<SeverityLevelDefaultError>,

    /// # Minimum float.
    ///
    /// Check if the float is greater than or equal to the minimum.
    ///
    /// ```rust
    /// float >= minimum
    /// ```
    ///
    pub float_minimum: Option<SeverityLevelDefaultError>,

    /// # Exclusive maximum float.
    ///
    /// Check if the float is less than the maximum.
    ///
    /// ```rust
    /// float < maximum
    /// ```
    ///
    pub float_exclusive_maximum: Option<SeverityLevelDefaultError>,

    /// # Exclusive minimum float.
    ///
    /// Check if the float is greater than the minimum.
    ///
    /// ```rust
    /// float > minimum
    /// ```
    ///
    pub float_exclusive_minimum: Option<SeverityLevelDefaultError>,

    /// # Multiple of float.
    ///
    /// Check if the float is a multiple of the value.
    ///
    /// ```rust
    /// float % multiple_of == 0
    /// ```
    ///
    pub float_multiple_of: Option<SeverityLevelDefaultError>,
}
