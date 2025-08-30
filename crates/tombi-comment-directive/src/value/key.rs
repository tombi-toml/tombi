use std::str::FromStr;

use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};
use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirective, WithCommonExtensibleRules};
use crate::TombiCommentDirectiveImpl;

pub type TombiKeyDirective = TombiValueDirective<KeyCommonExtensibleRules>;

pub type KeyCommonExtensibleRules = WithCommonExtensibleRules<KeyRules>;

impl TombiCommentDirectiveImpl for TombiKeyDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct KeyRules {
    /// # Key empty.
    ///
    /// Check if the key is empty.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// "" = true
    /// ```
    pub key_empty: Option<SeverityLevelDefaultWarn>,

    /// # Key required.
    ///
    /// Check if the key is required in this Table.
    pub key_required: Option<SeverityLevelDefaultError>,

    /// # Key not allowed.
    ///
    /// Check if the key is not defined in this Table.
    pub key_not_allowed: Option<SeverityLevelDefaultError>,

    /// # Key pattern.
    ///
    /// Check if the key matches the pattern in the Schema.
    pub key_pattern: Option<SeverityLevelDefaultError>,
}
