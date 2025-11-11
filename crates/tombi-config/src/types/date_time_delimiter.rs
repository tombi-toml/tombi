/// DateTime delimiter
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum DateTimeDelimiter {
    /// Example: `2021-01-01T00:00:00`
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "T"))]
    T,

    /// Example: `2021-01-01 00:00:00`
    Space,

    /// Preserve the source delimiter
    Preserve,
}
