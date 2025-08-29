use std::str::FromStr;

use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective};

pub type KeyTombiCommentDirective = ValueTombiCommentDirective<KeyRules>;

impl TombiCommentDirectiveImpl for KeyTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/key-tombi-directive.json").unwrap()
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

    /// Controls the severity level for key required errors
    pub key_required: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for key not allowed errors
    pub key_not_allowed: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for key pattern errors
    pub key_pattern: Option<SeverityLevelDefaultError>,
}
