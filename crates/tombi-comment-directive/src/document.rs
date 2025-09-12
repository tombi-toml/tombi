use std::str::FromStr;

use tombi_toml_version::TomlVersion;
use tombi_uri::SchemaUri;

use crate::TombiCommentDirectiveImpl;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/tombi-document-directive.json")))]
pub struct TombiDocumentDirectiveContent {
    /// # TOML version
    ///
    /// This directive specifies the TOML version of this document, with the highest priority.
    #[cfg_attr(feature = "jsonschema", schemars(default = "TomlVersion::default"))]
    pub toml_version: Option<TomlVersion>,

    /// # Formatter options
    pub format: Option<FormatOptions>,

    /// # Linter options
    pub lint: Option<LintOptions>,

    /// # Schema options
    pub schema: Option<SchemaOptions>,
}

impl TombiCommentDirectiveImpl for TombiDocumentDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-document-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct FormatOptions {
    /// # Format disabled
    ///
    /// If `true`, formatting is disabled for this document.
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_false"))]
    #[cfg_attr(feature = "jsonschema", schemars(extend("enum" = [true])))]
    disabled: Option<bool>,

    /// # Format disabled
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.disabled` instead.
    #[cfg_attr(feature = "jsonschema", deprecated)]
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_false"))]
    disable: Option<bool>,
}

impl FormatOptions {
    pub fn disabled(&self) -> Option<bool> {
        #[allow(deprecated)]
        self.disabled.or_else(|| self.disable)
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LintOptions {
    /// # Lint disabled
    ///
    /// If `true`, linting is disabled for this document.
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_false"))]
    #[cfg_attr(feature = "jsonschema", schemars(extend("enum" = [true])))]
    disabled: Option<bool>,

    /// # Lint disabled
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `lint.disabled` instead.
    #[cfg_attr(feature = "jsonschema", deprecated)]
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_false"))]
    disable: Option<bool>,
}

impl LintOptions {
    pub fn disabled(&self) -> Option<bool> {
        #[allow(deprecated)]
        self.disabled.or_else(|| self.disable)
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct SchemaOptions {
    /// # Enable strict schema validation
    ///
    /// If `additionalProperties` is not specified in the JSON Schema,
    /// the strict mode treats it as `additionalProperties: false`,
    /// which is different from the JSON Schema specification.
    #[cfg_attr(feature = "jsonschema", schemars(default = "crate::default_true"))]
    pub strict: Option<bool>,
}
