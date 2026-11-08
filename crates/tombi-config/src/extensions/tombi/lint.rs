use super::EnabledOnly;
use crate::SeverityLevel;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TombiLintFeatures {
    Enabled(EnabledOnly),
    Features(TombiLintFeatureTree),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, rename_all = "kebab-case")
)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TombiLintFeatureTree {
    /// # Tombi lint rules
    pub rules: Option<TombiLintRules>,
}

impl TombiLintFeatureTree {
    pub fn disabled() -> Self {
        Self {
            rules: Some(TombiLintRules::off()),
        }
    }
}

impl Default for TombiLintFeatures {
    fn default() -> Self {
        Self::Features(TombiLintFeatureTree::default())
    }
}

impl TombiLintFeatures {
    pub fn feature_tree(&self) -> TombiLintFeatureTree {
        match self {
            Self::Enabled(enabled) if !enabled.enabled.value() => TombiLintFeatureTree::disabled(),
            Self::Enabled(_) => TombiLintFeatureTree::default(),
            Self::Features(features) => *features,
        }
    }
}

/// Severity levels for diagnostics produced by the Tombi extension.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, rename_all = "kebab-case", default)
)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TombiLintRules {
    /// Report a schema catalog that does not exist: a local path that does not point to a file,
    /// or a URL for which the server answers 404 or 410.
    ///
    /// ```toml
    /// [schema.catalog]
    /// paths = ["./schemas/catalog.json"] # no file exists at this path
    /// ```
    pub missing_catalog_path: SeverityLevel,
    /// Report an `extends` entry that does not exist: a local path that does not point to a file,
    /// or a URL for which the server answers 404 or 410.
    ///
    /// ```toml
    /// extends = ["./base/tombi.toml"] # no file exists at this path
    /// ```
    pub missing_extends_path: SeverityLevel,
    /// Report a schema that does not exist: a local path that does not point to a file,
    /// or a URL for which the server answers 404 or 410.
    ///
    /// ```toml
    /// [[schemas]]
    /// path = "./schemas/app.json" # no file exists at this path
    /// ```
    pub missing_schema_path: SeverityLevel,
}

impl Default for TombiLintRules {
    fn default() -> Self {
        Self {
            missing_catalog_path: SeverityLevel::Error,
            missing_extends_path: SeverityLevel::Error,
            missing_schema_path: SeverityLevel::Error,
        }
    }
}

impl TombiLintRules {
    pub fn off() -> Self {
        Self {
            missing_catalog_path: SeverityLevel::Off,
            missing_extends_path: SeverityLevel::Off,
            missing_schema_path: SeverityLevel::Off,
        }
    }
}
