use tombi_severity_level::SeverityLevelDefaultError;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/integer-tombi-directive.json")))]
pub struct IntegerTombiCommentDirective {
    /// Controls the severity level for maximum integer errors
    pub maximum_integer: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum integer errors
    pub minimum_integer: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive maximum integer errors
    pub exclusive_maximum_integer: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive minimum integer errors
    pub exclusive_minimum_integer: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for multiple of integer errors
    pub multiple_of_integer: Option<SeverityLevelDefaultError>,
}
