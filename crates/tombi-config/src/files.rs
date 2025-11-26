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
    pub include: Option<Vec<String>>,

    /// # File patterns to exclude
    ///
    /// The file match pattern to exclude from formatting and linting.
    /// Supports glob pattern.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    #[cfg_attr(feature = "serde", serde(default = "default_exclude_patterns"))]
    pub exclude: Option<Vec<String>>,
}

fn default_include_patterns() -> Option<Vec<String>> {
    Some(vec!["**/*.toml".to_string()])
}

fn default_exclude_patterns() -> Option<Vec<String>> {
    // NOTE: For consistency between the VSCode extension and CLI,
    //       files that are considered TOML but do not have a *.toml extension
    //       are excluded by default.
    //
    //       See more details: https://github.com/tombi-toml/tombi/issues/1290
    Some(vec![
        "Cargo.lock".to_string(),
        "Gopkg.lock".to_string(),
        "Pipfile".to_string(),
        "pdm.lock".to_string(),
        "poetry.lock".to_string(),
        "uv.lock".to_string(),
    ])
}

impl Default for FilesOptions {
    fn default() -> Self {
        Self {
            include: default_include_patterns(),
            exclude: None,
        }
    }
}
