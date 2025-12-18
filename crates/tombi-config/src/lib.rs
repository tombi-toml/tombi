mod error;
mod files;
pub mod format;
mod level;
mod lint;
mod overrides;
mod schema;
mod server;
mod types;

pub use error::Error;
pub use files::FilesOptions;
pub use format::*;
pub use level::ConfigLevel;
pub use lint::*;
pub use overrides::*;
pub use schema::SchemaOverviewOptions;
pub use schema::{RootSchema, SchemaItem, SubSchema};
pub use server::{LspCompletion, LspOptions};
pub use tombi_severity_level::SeverityLevel;
pub use tombi_toml_version::TomlVersion;
pub use types::*;

pub const TOMBI_TOML_FILENAME: &str = "tombi.toml";
pub const CONFIG_TOML_FILENAME: &str = "config.toml";
pub const PYPROJECT_TOML_FILENAME: &str = "pyproject.toml";
pub const SUPPORTED_CONFIG_FILENAMES: [&str; 2] = [TOMBI_TOML_FILENAME, PYPROJECT_TOML_FILENAME];
pub const TOMBI_CONFIG_TOML_VERSION: TomlVersion = TomlVersion::V1_1_0;

/// # Tombi
///
/// **Tombi** (鳶 `/toɴbi/`) is a toolkit for TOML; providing a formatter/linter and language server.
/// See the [GitHub repository](https://github.com/tombi-toml/tombi) for more information.
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-toml-version" = TOMBI_CONFIG_TOML_VERSION)))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = concat!("https://", tombi_uri::schemastore_hostname!(), "/tombi.json"))))]
pub struct Config {
    /// # TOML version
    ///
    /// TOML version to use if not specified in the schema and comment directive.
    #[cfg_attr(feature = "jsonschema", schemars(default = "TomlVersion::default"))]
    pub toml_version: Option<TomlVersion>,

    pub files: Option<FilesOptions>,

    format: Option<FormatOptions>,

    lint: Option<LintOptions>,

    pub lsp: Option<LspOptions>,

    pub schema: Option<SchemaOverviewOptions>,

    /// # Schema items
    pub schemas: Option<Vec<SchemaItem>>,

    /// # Override config items
    overrides: Option<Vec<OverrideItem>>,
}

impl Config {
    pub fn include(&self) -> Option<&Vec<String>> {
        self.files.as_ref().and_then(|files| files.include.as_ref())
    }

    pub fn exclude(&self) -> Option<&Vec<String>> {
        self.files.as_ref().and_then(|files| files.exclude.as_ref())
    }

    pub fn overrides(&self) -> Option<&Vec<OverrideItem>> {
        self.overrides.as_ref()
    }

    pub fn format(&self, override_options: Option<&OverrideFormatOptions>) -> FormatOptions {
        let options = self.format.clone().unwrap_or_default();
        let base_rules = options.rules.unwrap_or_default();

        let rules = if let Some(override_rules) = override_options
            .as_ref()
            .and_then(|options| options.rules.as_ref())
        {
            base_rules.override_with(override_rules)
        } else {
            base_rules
        };

        FormatOptions { rules: Some(rules) }
    }

    pub fn lint(&self, override_options: Option<&OverrideLintOptions>) -> LintOptions {
        let base_rules = self
            .lint
            .clone()
            .and_then(|lint| lint.rules)
            .unwrap_or_default();

        let rules = if let Some(override_rules) = override_options
            .as_ref()
            .and_then(|options| options.rules.as_ref())
        {
            base_rules.override_with(override_rules)
        } else {
            base_rules
        };

        LintOptions { rules: Some(rules) }
    }
}
