use tombi_severity_level::SeverityLevelDefaultError;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/float-tombi-directive.json")))]
pub struct FloatTombiCommentDirective {
    /// Controls the severity level for type mismatch errors
    pub type_mismatch: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for const value errors
    pub const_value: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for enumerate value errors
    pub enumerate: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for maximum float errors
    pub maximum_float: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum float errors
    pub minimum_float: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive maximum float errors
    pub exclusive_maximum_float: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive minimum float errors
    pub exclusive_minimum_float: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for multiple of float errors
    pub multiple_of_float: Option<SeverityLevelDefaultError>,
}
