use std::str::FromStr;

use tombi_schema_store::SchemaUri;
use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};

use crate::TombiCommentDirectiveImpl;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct KeyTombiCommentDirectiveRules {
    /// # Dotted keys out of order.
    ///
    /// Check if dotted keys are defined out of order.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// apple.type = "fruit"
    /// orange.type = "fruit"
    /// apple.skin = "thin"
    /// orange.skin = "thick"
    ///
    /// # RECOMMENDED
    /// apple.type = "fruit"
    /// apple.skin = "thin"
    /// orange.type = "fruit"
    /// orange.skin = "thick"
    /// ```
    pub dotted_keys_out_of_order: Option<SeverityLevelDefaultWarn>,

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

impl TombiCommentDirectiveImpl for KeyTombiCommentDirectiveRules {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/key-tombi-directive.json").unwrap()
    }
}
