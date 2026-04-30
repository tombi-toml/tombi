use crate::GlobPattern;

/// # Files options
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Clone, PartialEq)]
pub struct FilesOptions {
    /// # File patterns to include
    ///
    /// The file match pattern to include in formatting and linting.
    /// Supports glob pattern.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    #[cfg_attr(feature = "serde", serde(default = "default_include_patterns"))]
    pub include: Option<Vec<GlobPattern>>,

    /// # File patterns to exclude
    ///
    /// The file match pattern to exclude from formatting and linting.
    /// Supports glob pattern.
    pub exclude: Option<Vec<GlobPattern>>,
}

fn default_include_patterns() -> Option<Vec<GlobPattern>> {
    Some(vec!["**/*.toml".into()])
}

impl Default for FilesOptions {
    fn default() -> Self {
        Self {
            include: default_include_patterns(),
            exclude: None,
        }
    }
}
