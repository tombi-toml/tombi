use tombi_severity_level::SeverityLevelDefaultError;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/key-tombi-directive.json")))]
pub struct KeyTombiCommentDirective {
    /// Controls the severity level for key required errors
    pub key_required: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for key not allowed errors
    pub key_not_allowed: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for pattern property errors
    pub pattern_property: Option<SeverityLevelDefaultError>,
}
