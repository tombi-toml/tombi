/// DateTime delimiter
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum DateTimeDelimiter {
    /// Example: `2021-01-01T00:00:00`
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "T"))]
    T,

    /// Example: `2021-01-01 00:00:00`
    #[cfg_attr(feature = "serde", serde(rename = "space"))]
    Space,

    /// Preserve the original delimiter
    #[cfg_attr(feature = "serde", serde(rename = "preserve"))]
    Preserve,
}
