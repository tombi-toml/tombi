use std::sync::Arc;

use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_severity_level::SeverityLevel;

use super::{DocumentSchema, SchemaUri};
use crate::{SchemaAccessor, SchemaAccessors};

pub type SubSchemaUriMap = tombi_hashmap::HashMap<Vec<SchemaAccessor>, SchemaUri>;
pub type DeprecatedLintLevels = tombi_hashmap::HashMap<SchemaUri, SeverityLevel>;

#[derive(Clone, Default)]
pub struct SourceSchema {
    pub root_schema: Option<Arc<DocumentSchema>>,
    pub sub_schema_uri_map: SubSchemaUriMap,
    pub deprecated_lint_levels: DeprecatedLintLevels,
    /// TOML version override from `[[schemas]]` config entry.
    ///
    /// Use [`toml_version()`](Self::toml_version) to get the resolved value.
    toml_version: Option<TomlVersion>,
}

impl SourceSchema {
    pub fn new(
        root_schema: Option<Arc<DocumentSchema>>,
        sub_schema_uri_map: SubSchemaUriMap,
        toml_version: Option<TomlVersion>,
        deprecated_lint_levels: DeprecatedLintLevels,
    ) -> Self {
        Self {
            root_schema,
            sub_schema_uri_map,
            deprecated_lint_levels,
            toml_version,
        }
    }

    pub fn insert_deprecated_lint_level(
        &mut self,
        schema_uri: SchemaUri,
        level: Option<SeverityLevel>,
    ) {
        if let Some(level) = level {
            self.deprecated_lint_levels.insert(schema_uri, level);
        }
    }

    /// Returns the resolved TOML version for this source.
    ///
    /// Priority: `[[schemas]]` config `toml-version` > JSON Schema `x-tombi-toml-version`.
    pub fn toml_version(&self) -> Option<TomlVersion> {
        self.toml_version.or_else(|| {
            self.root_schema
                .as_ref()
                .and_then(|root| root.toml_version())
        })
    }
}

impl std::fmt::Debug for SourceSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let root_schema_uri = self
            .root_schema
            .as_ref()
            .map(|schema| schema.schema_uri.to_string());
        let sub_schema_uri_map = self
            .sub_schema_uri_map
            .iter()
            .map(|(accessors, url)| {
                format!("[{:?}]: {}", SchemaAccessors::from(accessors.clone()), url)
            })
            .collect_vec()
            .join(", ");
        write!(
            f,
            "SourceSchema {{ root_schema: {root_schema_uri:?}, sub_schema_uri_map: {sub_schema_uri_map:?} }}"
        )
    }
}
