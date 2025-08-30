use std::str::FromStr;

use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirectiveContent, WithCommonRules, WithKeyRules};
use crate::TombiCommentDirectiveImpl;

pub type TombiKeyFloatDirectiveContent = TombiValueDirectiveContent<KeyFloatCommonRules>;

pub type TombiFloatDirectiveContent = TombiValueDirectiveContent<FloatCommonRules>;

pub type KeyFloatCommonRules = WithKeyRules<WithCommonRules<FloatRules>>;

pub type FloatCommonRules = WithCommonRules<FloatRules>;

impl TombiCommentDirectiveImpl for TombiKeyFloatDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-float-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiFloatDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-float-directive.json").unwrap()
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
