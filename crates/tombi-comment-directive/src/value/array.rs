use tombi_severity_level::SeverityLevelDefaultError;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/array-tombi-directive.json")))]
pub struct ArrayTombiCommentDirective {
    /// Controls the severity level for type mismatch errors
    pub type_mismatch: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for const value errors
    pub const_value: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for enumerate value errors
    pub enumerate: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for max values errors
    pub max_values: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for min values errors
    pub min_values: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for unique values errors
    pub unique_values: Option<SeverityLevelDefaultError>,
}
