use tombi_severity_level::SeverityLevelDefaultWarn;
use tombi_toml_version::TomlVersion;
use tombi_x_keyword::{ArrayValuesOrder, TableKeysOrder};

use crate::{
    BoolDefaultTrue, JSON_SCHEMASTORE_CATALOG_URL, SchemaCatalogPath, TOMBI_SCHEMASTORE_CATALOG_URL,
};

/// # Schema overview options
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaOverviewOptions {
    /// # Enable the schema validation
    pub enabled: Option<BoolDefaultTrue>,

    /// # Enable strict schema validation
    ///
    /// If `additionalProperties` is not specified in the JSON Schema,
    /// the strict mode treats it as `additionalProperties: false`,
    /// which is different from the JSON Schema specification.
    pub strict: Option<BoolDefaultTrue>,

    /// # Schema catalog options
    pub catalog: Option<SchemaCatalog>,
}

impl SchemaOverviewOptions {
    pub const fn default() -> Self {
        Self {
            enabled: None,
            strict: None,
            catalog: None,
        }
    }

    pub fn catalog_paths(&self) -> Option<Vec<SchemaCatalogPath>> {
        if self.enabled.unwrap_or_default().value() {
            self.catalog
                .clone()
                .unwrap_or_default()
                .paths
                .as_ref()
                .map(|path| path.to_vec())
        } else {
            None
        }
    }

    pub fn strict(&self) -> Option<bool> {
        self.strict.as_ref().map(|strict| strict.value())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaCatalog {
    /// # The schema catalog path/url array
    ///
    /// The catalog is evaluated after the schemas specified by [[schemas]].\
    /// Schemas are loaded in order from the beginning of the catalog list.
    #[cfg_attr(feature = "jsonschema", schemars(default = "catalog_paths_default"))]
    #[cfg_attr(feature = "serde", serde(default = "catalog_paths_default"))]
    pub paths: Option<Vec<SchemaCatalogPath>>,
}

impl Default for SchemaCatalog {
    fn default() -> Self {
        Self {
            paths: catalog_paths_default(),
        }
    }
}

fn catalog_paths_default() -> Option<Vec<SchemaCatalogPath>> {
    Some(vec![
        TOMBI_SCHEMASTORE_CATALOG_URL.into(),
        JSON_SCHEMASTORE_CATALOG_URL.into(),
    ])
}

/// # Schema item
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaItem {
    Root(RootSchema),
    Sub(SubSchema),
}

impl SchemaItem {
    pub fn path(&self) -> &str {
        match self {
            Self::Root(item) => &item.path,
            Self::Sub(item) => &item.path,
        }
    }

    pub fn include(&self) -> &[String] {
        match self {
            Self::Root(item) => &item.include,
            Self::Sub(item) => &item.include,
        }
    }

    pub fn toml_version(&self) -> Option<TomlVersion> {
        match self {
            Self::Root(item) => item.toml_version,
            Self::Sub(_) => None,
        }
    }

    pub fn root(&self) -> Option<&str> {
        match self {
            Self::Root(_) => None,
            Self::Sub(item) => Some(&item.root),
        }
    }

    pub fn deprecated_lint_level(&self) -> Option<SeverityLevelDefaultWarn> {
        match self {
            Self::Root(item) => item.deprecated_lint_level(),
            Self::Sub(item) => item.deprecated_lint_level(),
        }
    }

    pub fn format(&self) -> Option<&SchemaFormatOptions> {
        match self {
            Self::Root(item) => item.format.as_ref(),
            Self::Sub(item) => item.format.as_ref(),
        }
    }

    pub fn overrides(&self) -> Option<&Vec<SchemaOverrideItem>> {
        match self {
            Self::Root(item) => item.overrides.as_ref(),
            Self::Sub(item) => item.overrides.as_ref(),
        }
    }
}

/// # The schema for the root table
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Clone, PartialEq)]
pub struct RootSchema {
    /// # The TOML version that the schema is available
    pub toml_version: Option<TomlVersion>,

    /// # The schema path
    pub path: String,

    /// # The file match pattern of the schema
    ///
    /// The file match pattern to include the target to apply the schema.
    /// Supports glob pattern.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    pub include: Vec<String>,

    /// # Schema-specific format options
    pub format: Option<SchemaFormatOptions>,

    /// # Schema-specific lint options
    pub lint: Option<SchemaLintOptions>,

    /// # Schema-specific overrides
    pub overrides: Option<Vec<SchemaOverrideItem>>,
}

impl RootSchema {
    pub fn deprecated_lint_level(&self) -> Option<SeverityLevelDefaultWarn> {
        self.lint
            .as_ref()
            .and_then(|lint| lint.deprecated_lint_level())
    }
}

/// # The schema for the sub value
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Clone, PartialEq)]
pub struct SubSchema {
    /// # The accessors to apply the sub schema
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    #[cfg_attr(feature = "jsonschema", schemars(example = "tool.tombi"))]
    #[cfg_attr(feature = "jsonschema", schemars(example = "items[0].name"))]
    pub root: String,

    /// # The sub schema path
    pub path: String,

    /// # The file match pattern of the sub schema
    ///
    /// The file match pattern to include the target to apply the sub schema.
    /// Supports glob pattern.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    pub include: Vec<String>,

    /// # Schema-specific format options
    pub format: Option<SchemaFormatOptions>,

    /// # Schema-specific lint options
    pub lint: Option<SchemaLintOptions>,

    /// # Schema-specific overrides
    pub overrides: Option<Vec<SchemaOverrideItem>>,
}

impl SubSchema {
    pub fn deprecated_lint_level(&self) -> Option<SeverityLevelDefaultWarn> {
        self.lint
            .as_ref()
            .and_then(|lint| lint.deprecated_lint_level())
    }
}

/// # Schema-specific lint options
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaLintOptions {
    /// # Schema-specific lint rules
    pub rules: Option<SchemaLintRules>,
}

impl SchemaLintOptions {
    pub fn deprecated_lint_level(&self) -> Option<SeverityLevelDefaultWarn> {
        self.rules.as_ref().and_then(|rules| rules.deprecated)
    }
}

/// # Schema-specific lint rules
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaLintRules {
    /// # Deprecated
    ///
    /// Override the deprecated diagnostic level for this schema.
    pub deprecated: Option<SeverityLevelDefaultWarn>,
}

/// # Schema-specific format options
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaFormatOptions {
    /// # Schema-specific format rules
    pub rules: Option<SchemaFormatRules>,
}

/// # Schema-specific format rules
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaFormatRules {
    /// # Whether schema-defined array values ordering is enabled
    pub array_values_order: Option<SchemaArrayValuesOrderRule>,

    /// # Whether schema-defined table key ordering is enabled
    pub table_keys_order: Option<SchemaTableKeysOrderRule>,
}

/// # Schema-defined array values ordering
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaArrayValuesOrderRule {
    /// # Whether schema-defined array values ordering is enabled
    pub enabled: Option<BoolDefaultTrue>,
}

/// # Schema-defined table key ordering
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaTableKeysOrderRule {
    /// # Whether schema-defined table key ordering is enabled
    pub enabled: Option<BoolDefaultTrue>,
}

/// # Accessor pattern
///
/// To apply it to the Root Table, use `""`.
///
/// Array indices are matched as wildcards. That means `[*]` and numeric
/// indices such as `[0]` are treated the same and will match any array
/// element, so `items[0].name` behaves like `items[*].name`.
///
/// **Example**:
///   - `""`
///   - `"tool.*"`
///   - `"items[*].name"`
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct PatternAccessor(
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 0)))] pub String,
);

impl std::ops::Deref for PatternAccessor {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for PatternAccessor {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// # Schema-specific override item
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaOverrideItem {
    /// # Accessor patterns to override
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    pub targets: Vec<PatternAccessor>,

    /// # Format options to override
    pub format: Option<SchemaOverrideFormatOptions>,
}

/// # Schema-specific override format options
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaOverrideFormatOptions {
    /// # Schema-specific override format rules
    pub rules: Option<SchemaOverrideFormatRules>,
}

/// # Schema-specific override format rules
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaOverrideFormatRules {
    /// # Override array values ordering for matched roots
    pub array_values_order: Option<SchemaOverrideArrayValuesOrderRule>,

    /// # Override table key ordering for matched roots
    pub table_keys_order: Option<SchemaOverrideTableKeysOrderRule>,
}

/// # Override array values ordering for matched roots
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaOverrideArrayValuesOrderRule {
    Order(ArrayValuesOrder),
    Rule(SchemaArrayValuesOrderRule),
}

impl SchemaOverrideArrayValuesOrderRule {
    pub fn enabled(&self) -> Option<BoolDefaultTrue> {
        match self {
            Self::Order(_) => None,
            Self::Rule(rule) => rule.enabled,
        }
    }

    pub fn order(&self) -> Option<ArrayValuesOrder> {
        match self {
            Self::Order(order) => Some(*order),
            Self::Rule(_) => None,
        }
    }
}

/// # Override table key ordering for matched roots
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaOverrideTableKeysOrderRule {
    Order(TableKeysOrder),
    Rule(SchemaTableKeysOrderRule),
}

impl SchemaOverrideTableKeysOrderRule {
    pub fn enabled(&self) -> Option<BoolDefaultTrue> {
        match self {
            Self::Order(_) => None,
            Self::Rule(rule) => rule.enabled,
        }
    }

    pub fn order(&self) -> Option<TableKeysOrder> {
        match self {
            Self::Order(order) => Some(*order),
            Self::Rule(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::JSON_SCHEMASTORE_CATALOG_URL;

    use super::*;

    #[test]
    fn schema_catalog_paths_default() {
        let schema = SchemaOverviewOptions::default();
        let expected = Some(vec![
            TOMBI_SCHEMASTORE_CATALOG_URL.into(),
            JSON_SCHEMASTORE_CATALOG_URL.into(),
        ]);
        let default_paths = schema.catalog_paths();

        pretty_assertions::assert_eq!(default_paths, expected);
    }

    #[test]
    fn schema_catalog_paths_empty() {
        let schema = SchemaOverviewOptions {
            catalog: Some(SchemaCatalog {
                paths: Some(vec![]),
            }),
            ..Default::default()
        };

        let expected: Vec<SchemaCatalogPath> = vec![];
        pretty_assertions::assert_eq!(schema.catalog_paths(), Some(expected));
    }
}
