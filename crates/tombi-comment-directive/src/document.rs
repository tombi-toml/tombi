use std::str::FromStr;

use tombi_schema_store::{DocumentSchema, SchemaUri};
use tombi_toml_version::TomlVersion;

use crate::{schema_store, DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URI};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/document-tombi-directive.json")))]
pub struct TombiDocumentCommentDirective {
    /// # TOML version.
    ///
    /// This directive specifies the TOML version of this document, with the highest priority.
    #[cfg_attr(feature = "jsonschema", schemars(default = "TomlVersion::default"))]
    pub toml_version: Option<TomlVersion>,

    /// # Formatter options.
    pub format: Option<FormatOptions>,

    /// # Linter options.
    pub lint: Option<LintOptions>,

    /// # Schema options.
    pub schema: Option<SchemaOptions>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct FormatOptions {
    /// # Format disable
    ///
    /// Disable formatting for this document.
    #[cfg_attr(feature = "jsonschema", schemars(default = "default_false"))]
    pub disable: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LintOptions {
    /// # Lint disable
    ///
    /// Disable linting for this document.
    #[cfg_attr(feature = "jsonschema", schemars(default = "default_false"))]
    pub disable: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct SchemaOptions {
    /// # Enable strict schema validation.
    ///
    /// If `additionalProperties` is not specified in the JSON Schema,
    /// the strict mode treats it as `additionalProperties: false`,
    /// which is different from the JSON Schema specification.
    #[cfg_attr(feature = "jsonschema", schemars(default = "default_true"))]
    pub strict: Option<bool>,
}

#[cfg(feature = "jsonschema")]
#[allow(unused)]
#[inline]
fn default_true() -> Option<bool> {
    Some(true)
}

#[cfg(feature = "jsonschema")]
#[allow(unused)]
#[inline]
fn default_false() -> Option<bool> {
    Some(false)
}

#[inline]
pub fn document_comment_directive_schema_uri() -> &'static SchemaUri {
    DOCUMENT_COMMENT_DIRECTIVE_SCHEMA_URI.get_or_init(|| {
        SchemaUri::from_str("tombi://json.tombi.dev/document-tombi-directive.json").unwrap()
    })
}

pub async fn document_comment_directive_document_schema() -> DocumentSchema {
    let schema_store = schema_store().await;
    let schema_uri = document_comment_directive_schema_uri();
    let tombi_json::ValueNode::Object(object) = schema_store
        .fetch_schema_value(schema_uri)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!(
            "Failed to fetch document comment directive schema from URL '{schema_uri}'. \
             The fetched value was not an object."
        );
    };
    DocumentSchema::new(object, schema_uri.clone())
}
