mod error;
mod files;
pub mod format;
mod level;
mod lint;
mod schema;
mod server;
mod types;

pub use error::Error;
pub use files::FilesOptions;
pub use format::FormatOptions;
pub use level::ConfigLevel;
pub use lint::{LintOptions, SeverityLevel};
pub use schema::SchemaOptions;
pub use schema::{RootSchema, Schema, SubSchema};
pub use server::{LspCompletion, LspOptions};
pub use tombi_toml_version::TomlVersion;
pub use types::*;

pub const TOMBI_TOML_FILENAME: &str = "tombi.toml";
pub const CONFIG_TOML_FILENAME: &str = "config.toml";
pub const PYPROJECT_TOML_FILENAME: &str = "pyproject.toml";
pub const SUPPORTED_CONFIG_FILENAMES: [&str; 2] = [TOMBI_TOML_FILENAME, PYPROJECT_TOML_FILENAME];
pub const TOMBI_CONFIG_TOML_VERSION: TomlVersion = TomlVersion::V1_1_0_Preview;

/// # Tombi
///
/// **Tombi** (鳶) is a toolkit for TOML; providing a formatter/linter and language server.
/// See the [GitHub repository](https://github.com/tombi-toml/tombi) for more information.
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-toml-version" = TOMBI_CONFIG_TOML_VERSION)))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "https://json.schemastore.org/tombi.json")))]
pub struct Config {
    /// # TOML version.
    ///
    /// TOML version to use if not specified in the schema.
    #[cfg_attr(feature = "jsonschema", schemars(default = "TomlVersion::default"))]
    pub toml_version: Option<TomlVersion>,

    /// # Deprecated. Use `files.include` instead.
    #[cfg_attr(feature = "jsonschema", deprecated)]
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    include: Option<Vec<String>>,

    /// # Deprecated. Use `files.exclude` instead.
    #[cfg_attr(feature = "jsonschema", deprecated)]
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    exclude: Option<Vec<String>>,

    pub files: Option<FilesOptions>,

    /// # Formatter options.
    pub format: Option<FormatOptions>,

    /// # Linter options.
    pub lint: Option<LintOptions>,

    /// # Language Server Protocol options.
    lsp: Option<LspOptions>,

    /// # Deprecated. Use `lsp` instead.
    #[cfg_attr(feature = "jsonschema", deprecated)]
    server: Option<LspOptions>,

    /// # Schema options.
    pub schema: Option<SchemaOptions>,

    /// # Schema catalog items.
    pub schemas: Option<Vec<Schema>>,
}

impl Config {
    pub fn include(&self) -> Option<&Vec<String>> {
        #[allow(deprecated)]
        self.files
            .as_ref()
            .and_then(|files| files.include.as_ref())
            .or(self.include.as_ref())
    }

    pub fn exclude(&self) -> Option<&Vec<String>> {
        #[allow(deprecated)]
        self.files
            .as_ref()
            .and_then(|files| files.exclude.as_ref())
            .or(self.exclude.as_ref())
    }

    pub fn lsp(&self) -> Option<&LspOptions> {
        #[allow(deprecated)]
        self.lsp.as_ref().or(self.server.as_ref())
    }
}
