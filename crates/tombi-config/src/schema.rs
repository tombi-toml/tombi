use tombi_toml_version::TomlVersion;

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
