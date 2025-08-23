use tombi_severity_level::SeverityLevelDefaultError;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/string-tombi-directive.json")))]
pub struct StringTombiCommentDirective {
    /// Controls the severity level for maximum length errors
    pub maximum_length: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum length errors
    pub minimum_length: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for format errors
    pub format: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for pattern errors
    pub pattern: Option<SeverityLevelDefaultError>,
}
