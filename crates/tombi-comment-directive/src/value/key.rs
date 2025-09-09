use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    ErrorRuleOptions, TombiValueDirectiveContent, WarnRuleOptions, WithCommonExtensibleRules,
};
use crate::TombiCommentDirectiveImpl;

pub type KeyCommonExtensibleRules = WithCommonExtensibleRules<KeyRules>;

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyCommonExtensibleRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct KeyRules {
    /// # Key empty
    ///
    /// Check if the key is empty.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// "" = true
    /// ```
    ///
    pub key_empty: Option<WarnRuleOptions>,

    /// # Key not allowed
    ///
    /// Check if the key is not defined in this Table.
    ///
    pub key_not_allowed: Option<ErrorRuleOptions>,

    /// # Key pattern
    ///
    /// Check if the key matches the pattern in the Schema.
    ///
    pub key_pattern: Option<ErrorRuleOptions>,
}
