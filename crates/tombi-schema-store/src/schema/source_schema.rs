use std::sync::Arc;

use itertools::Itertools;
use tombi_config::{SchemaFormatRules, SchemaLintRules, TomlVersion};
use tombi_severity_level::SeverityLevelDefaultWarn;

use super::{DocumentSchema, SchemaOverrides, SchemaUri};
use crate::{PatternAccessor, PatternAccessors};

#[derive(Debug, Clone)]
pub struct SubSchemaLink {
    pub schema_uri: SchemaUri,
    /// Effective sub-schema-level strict setting after lower-priority fallback resolution.
    pub strict: bool,
}

pub type SubSchemaLinkMap = tombi_hashmap::IndexMap<Vec<PatternAccessor>, SubSchemaLink>;
pub type SchemaFormatRulesMap = tombi_hashmap::HashMap<SchemaUri, SchemaFormatRules>;
pub type SchemaLintRulesMap = tombi_hashmap::HashMap<SchemaUri, SchemaLintRules>;
pub type SchemaOverridesMap = tombi_hashmap::HashMap<SchemaUri, SchemaOverrides>;

#[derive(Clone, Default)]
pub struct SourceSchema {
    pub root_schema: Option<Arc<DocumentSchema>>,
    pub sub_schema_link_map: SubSchemaLinkMap,
    pub deprecated_lint_level: Option<SeverityLevelDefaultWarn>,
    pub schema_format_rules: SchemaFormatRulesMap,
    pub schema_lint_rules: SchemaLintRulesMap,
    pub schema_overrides: SchemaOverridesMap,
    /// TOML version override from `[[schemas]]` config entry.
    ///
    /// Use [`toml_version()`](Self::toml_version) to get the resolved value.
    toml_version: Option<TomlVersion>,
}

impl SourceSchema {
    pub fn new(
        root_schema: Option<Arc<DocumentSchema>>,
        sub_schema_link_map: SubSchemaLinkMap,
        toml_version: Option<TomlVersion>,
        deprecated_lint_level: Option<SeverityLevelDefaultWarn>,
        schema_format_rules: SchemaFormatRulesMap,
        schema_lint_rules: SchemaLintRulesMap,
        schema_overrides: SchemaOverridesMap,
    ) -> Self {
        Self {
            root_schema,
            sub_schema_link_map,
            deprecated_lint_level,
            schema_format_rules,
            schema_lint_rules,
            schema_overrides,
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
        let sub_schema_link_map = self
            .sub_schema_link_map
            .iter()
            .map(|(accessors, link)| {
                format!(
                    "[{:?}]: {} (strict: {})",
                    PatternAccessors::from(accessors.clone()),
                    link.schema_uri,
                    link.strict,
                )
            })
            .collect_vec()
            .join(", ");
        write!(
            f,
            "SourceSchema {{ root_schema: {root_schema_uri:?}, sub_schema_link_map: {sub_schema_link_map:?} }}"
        )
    }
}
