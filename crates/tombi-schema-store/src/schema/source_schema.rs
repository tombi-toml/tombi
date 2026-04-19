use std::sync::Arc;

use tombi_config::TomlVersion;
use tombi_severity_level::SeverityLevelDefaultWarn;

use super::{DocumentSchema, SchemaOverrides, SchemaUri, TableOrderOverride};
use crate::RootAccessor;

#[derive(Clone, Debug, Default)]
pub struct SourceSchema {
    pub root_schema: Option<Arc<DocumentSchema>>,
    pub sub_schema_map: SourceSubSchemaMap,
    pub deprecated_lint_level: Option<SeverityLevelDefaultWarn>,
    pub array_values_order_enabled: bool,
    pub table_keys_order_enabled: bool,
    pub overrides: SchemaOverrides,
    /// TOML version override from `[[schemas]]` config entry.
    ///
    /// Use [`toml_version()`](Self::toml_version) to get the resolved value.
    toml_version: Option<TomlVersion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSubSchema {
    pub schema_uri: SchemaUri,
}

pub type SourceSubSchemaMap = tombi_hashmap::IndexMap<Vec<RootAccessor>, SourceSubSchema>;

impl SourceSchema {
    pub fn new(
        root_schema: Option<Arc<DocumentSchema>>,
        sub_schema_map: SourceSubSchemaMap,
        toml_version: Option<TomlVersion>,
        deprecated_lint_level: Option<SeverityLevelDefaultWarn>,
        array_values_order_enabled: bool,
        table_keys_order_enabled: bool,
    ) -> Self {
        Self {
            root_schema,
            sub_schema_map,
            array_values_order_enabled,
            deprecated_lint_level,
            table_keys_order_enabled,
            overrides: Default::default(),
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

    pub fn push_table_order_override(
        &mut self,
        target: Vec<RootAccessor>,
        table_order_override: TableOrderOverride,
    ) {
        self.overrides.table_keys_order.push_schema_override(
            target,
            table_order_override.disabled,
            table_order_override.order,
        );
    }
}
