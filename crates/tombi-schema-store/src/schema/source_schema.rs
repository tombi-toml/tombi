use std::sync::Arc;

use itertools::Itertools;
use tombi_config::TomlVersion;

use super::{DocumentSchema, SchemaUri};
use crate::{SchemaAccessor, SchemaAccessors};

pub type SubSchemaUriMap = tombi_hashmap::HashMap<Vec<SchemaAccessor>, SchemaUri>;

#[derive(Clone, Default)]
pub struct SourceSchema {
    pub root_schema: Option<Arc<DocumentSchema>>,
    pub sub_schema_uri_map: SubSchemaUriMap,
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
    ) -> Self {
        Self {
            root_schema,
            sub_schema_uri_map,
            toml_version,
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
