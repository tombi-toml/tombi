//! Formatting options
//!
//! Options for adjusting the formatting of TOML files.
//! Initially, this structure contained settings related to `line-width`, etc.,
//! but to avoid unnecessary discussions about the format, all settings have been moved to [formatter::FormatDefinition].
//! In the future, there is a possibility that options will be added to this structure,
//! but considering the recent trend of formatters to avoid such discussions by restricting the settings and its results,
//! this structure is currently empty.

/// # Formatter options
///
/// To avoid needless discussion of formatting rules,
/// we do not currently have a configuration item for formatting.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Clone, PartialEq)]
pub struct FormatOptions {}

impl FormatOptions {
    pub const fn default() -> Self {
        Self {}
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self::default()
    }
}
