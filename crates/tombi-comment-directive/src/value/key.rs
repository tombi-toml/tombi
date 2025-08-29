use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};

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
