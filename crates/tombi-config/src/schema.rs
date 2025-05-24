use tombi_toml_version::TomlVersion;

use crate::{BoolDefaultTrue, OneOrMany, SchemaCatalogPath};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaOptions {
    /// # Enable or disable the schema.
    pub enabled: Option<BoolDefaultTrue>,

    /// # Enable or disable strict schema validation.
    ///
    /// If `additionalProperties` is not specified in the JSON Schema,
    /// the strict mode treats it as `additionalProperties: false`,
    /// which is different from the JSON Schema specification.
    pub strict: Option<BoolDefaultTrue>,

    /// # Schema catalog options.
    pub catalog: Option<SchemaCatalog>,
}

impl SchemaOptions {
    pub const fn default() -> Self {
        Self {
            enabled: None,
            strict: None,
            catalog: None,
        }
    }

    pub fn catalog_paths(&self) -> Option<Vec<SchemaCatalogPath>> {
        if self.enabled.unwrap_or_default().value() {
            Some(
                self.catalog
                    .as_ref()
                    .and_then(|catalog| catalog.path.as_ref().map(|path| path.as_ref().to_vec()))
                    .unwrap_or_else(|| vec![SchemaCatalogPath::default()]),
            )
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaCatalog {
    /// # The schema catalog path or url.
    ///
    /// The catalog is evaluated after the schemas specified by [[schemas]].
    ///
    /// You can specify multiple catalogs by making it an array.
    /// If you specify an array, the catalogs are searched in order of priority starting from the first catalog.
    /// If you want to disable the default catalog, specify an empty array.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "SchemaCatalogPath::default")
    )]
    pub path: Option<OneOrMany<SchemaCatalogPath>>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq)]
pub enum Schema {
    Root(RootSchema),
    Sub(SubSchema),
    OldSub(OldSubSchema),
}

impl Schema {
    pub fn path(&self) -> &str {
        match self {
            Self::Root(item) => &item.path,
            Self::Sub(item) => &item.path,
            Self::OldSub(item) => &item.path,
        }
    }

    pub fn include(&self) -> &[String] {
        match self {
            Self::Root(item) => &item.include,
            Self::Sub(item) => &item.include,
            Self::OldSub(item) => &item.include,
        }
    }

    pub fn toml_version(&self) -> Option<TomlVersion> {
        match self {
            Self::Root(item) => item.toml_version,
            Self::Sub(_) => None,
            Self::OldSub(_) => None,
        }
    }

    pub fn root(&self) -> Option<&str> {
        match self {
            Self::Root(_) => None,
            Self::Sub(item) => item.root.as_deref(),
            #[allow(deprecated)]
            Self::OldSub(item) => item.root_keys.as_deref(),
        }
    }
}

/// # The schema for the root table.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Clone, PartialEq)]
pub struct RootSchema {
    /// # The TOML version that the schema is available.
    pub toml_version: Option<TomlVersion>,

    /// # The schema path.
    pub path: String,

    /// # The file match pattern of the schema.
    ///
    /// The file match pattern to include the target to apply the schema.
    /// Supports glob pattern.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    pub include: Vec<String>,
}

/// # The schema for the sub value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Clone, PartialEq)]
pub struct SubSchema {
    /// # The accessors to apply the sub schema.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    #[cfg_attr(feature = "jsonschema", schemars(example = "tools.tombi"))]
    #[cfg_attr(feature = "jsonschema", schemars(example = "items[0].name"))]
    pub root: Option<String>,

    /// # The sub schema path.
    pub path: String,

    /// # The file match pattern of the sub schema.
    ///
    /// The file match pattern to include the target to apply the sub schema.
    /// Supports glob pattern.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    pub include: Vec<String>,
}

/// # The schema for the old sub value.
///
/// This is for backward compatibility.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Clone, PartialEq)]
pub struct OldSubSchema {
    /// # The sub schema path.
    pub path: String,

    /// # The file match pattern of the sub schema.
    ///
    /// The file match pattern to include the target to apply the sub schema.
    /// Supports glob pattern.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    pub include: Vec<String>,

    /// # The keys to apply the sub schema.
    ///
    /// Please use `root` instead.
    #[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub root_keys: Option<String>,
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn schema_catalog_paths_default() {
        let schema = SchemaOptions::default();
        let _expected = [SchemaCatalogPath::default()];

        assert_matches!(schema.catalog_paths(), Some(_expected));
    }

    #[test]
    fn schema_catalog_paths_empty() {
        let schema = SchemaOptions {
            catalog: Some(SchemaCatalog {
                path: Some(vec![].into()),
            }),
            ..Default::default()
        };

        let _expected: Vec<SchemaCatalogPath> = vec![];

        assert_matches!(schema.catalog_paths(), Some(_expected));
    }
}
