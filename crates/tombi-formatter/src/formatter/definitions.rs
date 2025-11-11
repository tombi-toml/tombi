/// FormatDefinitions provides the definition of the format that does not have the freedom set by [`FormatOptions`][crate::FormatOptions].
///
/// NOTE: Some of the items defined in FormatDefinitions may be moved to [`FormatOptions`][crate::FormatOptions] in the future.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, Copy)]
pub struct FormatDefinitions {}

impl FormatDefinitions {}
