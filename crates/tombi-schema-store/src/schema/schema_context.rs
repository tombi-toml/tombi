use tombi_severity_level::SeverityLevelDefaultWarn;
use tombi_x_keyword::StringFormat;

use crate::schema::schema_cycle_guard::SchemaVisits;
use crate::{
    Accessor, ArrayOrderOverride, ArrayOrderOverrides, CurrentSchema, SchemaUri,
    TableOrderOverride, TableOrderOverrides, ValueSchema, XTombiArrayValuesOrder,
    XTombiTableKeysOrder,
};

#[derive(Default)]
pub struct SchemaContextOverrides<'a> {
    pub array_values_order: Option<&'a ArrayOrderOverrides>,
    pub table_keys_order: Option<&'a TableOrderOverrides>,
}

pub struct SchemaContext<'a> {
    pub toml_version: tombi_config::TomlVersion,
    pub root_schema: Option<&'a crate::DocumentSchema>,
    pub sub_schema_map: Option<&'a crate::SourceSubSchemaMap>,
    pub deprecated_lint_level: Option<SeverityLevelDefaultWarn>,
    pub overrides: SchemaContextOverrides<'a>,
    pub schema_visits: SchemaVisits,
    pub store: &'a crate::SchemaStore,
    pub strict: Option<bool>,
}

impl SchemaContext<'_> {
    pub fn from_source_schema<'a>(
        toml_version: tombi_config::TomlVersion,
        source_schema: Option<&'a crate::SourceSchema>,
        store: &'a crate::SchemaStore,
        strict: Option<bool>,
    ) -> SchemaContext<'a> {
        SchemaContext {
            toml_version,
            root_schema: source_schema.and_then(|schema| schema.root_schema.as_deref()),
            sub_schema_map: source_schema.map(|schema| &schema.sub_schema_map),
            deprecated_lint_level: source_schema.and_then(|schema| schema.deprecated_lint_level),
            overrides: SchemaContextOverrides {
                array_values_order: source_schema
                    .map(|schema| &schema.overrides.array_values_order),
                table_keys_order: source_schema.map(|schema| &schema.overrides.table_keys_order),
            },
            schema_visits: Default::default(),
            store,
            strict,
        }
    }

    #[inline]
    pub fn strict(&self) -> bool {
        self.strict.unwrap_or_else(|| self.store.strict())
    }

    pub fn with_strict(&self, strict: Option<bool>) -> SchemaContext<'_> {
        SchemaContext {
            toml_version: self.toml_version,
            root_schema: self.root_schema,
            sub_schema_map: self.sub_schema_map,
            deprecated_lint_level: self.deprecated_lint_level,
            overrides: SchemaContextOverrides {
                array_values_order: self.overrides.array_values_order,
                table_keys_order: self.overrides.table_keys_order,
            },
            schema_visits: self.schema_visits.clone(),
            store: self.store,
            strict,
        }
    }

    #[inline]
    pub fn has_string_format(&self, format: StringFormat) -> bool {
        self.root_schema
            .and_then(|root| root.string_formats())
            .is_some_and(|formats| formats.contains(&format))
    }

    #[inline]
    pub fn deprecated_lint_level(&self) -> Option<SeverityLevelDefaultWarn> {
        self.deprecated_lint_level
    }

    pub async fn table_keys_order_enabled(
        &self,
        accessors: &[Accessor],
        schema_uri: &SchemaUri,
    ) -> bool {
        if let Some(override_item) = self.table_order_override(accessors) {
            return !override_item.disabled;
        }

        let mut normalized = schema_uri.clone();
        normalized.set_fragment(None);

        if self.root_schema.is_some_and(|root_schema| {
            let mut root_schema_uri = root_schema.schema_uri.clone();
            root_schema_uri.set_fragment(None);
            root_schema_uri == normalized
        }) {
            return self
                .store
                .table_keys_order_enabled_for_schema(&normalized, None)
                .await
                .unwrap_or(true);
        }

        let sub_root_accessors = self.sub_schema_map.and_then(|sub_schema_map| {
            sub_schema_map
                .iter()
                .find_map(|(sub_root_accessors, sub_schema)| {
                    let mut sub_schema_uri = sub_schema.schema_uri.clone();
                    sub_schema_uri.set_fragment(None);
                    (sub_schema_uri == normalized
                        && sub_root_accessors.len() <= accessors.len()
                        && sub_root_accessors
                            .iter()
                            .zip(accessors.iter())
                            .all(|(expected, actual)| expected == actual))
                    .then_some(sub_root_accessors.as_slice())
                })
        });

        self.store
            .table_keys_order_enabled_for_schema(&normalized, sub_root_accessors)
            .await
            .unwrap_or(true)
    }

    pub fn table_order_override(&self, accessors: &[Accessor]) -> Option<&TableOrderOverride> {
        self.overrides
            .table_keys_order
            .and_then(|overrides| overrides.get(accessors))
    }

    pub async fn array_values_order_enabled(
        &self,
        accessors: &[Accessor],
        schema_uri: &SchemaUri,
    ) -> bool {
        if let Some(override_item) = self.array_order_override(accessors) {
            return !override_item.disabled;
        }

        let mut normalized = schema_uri.clone();
        normalized.set_fragment(None);

        if self.root_schema.is_some_and(|root_schema| {
            let mut root_schema_uri = root_schema.schema_uri.clone();
            root_schema_uri.set_fragment(None);
            root_schema_uri == normalized
        }) {
            return self
                .store
                .array_values_order_enabled_for_schema(&normalized, None)
                .await
                .unwrap_or(true);
        }

        let sub_root_accessors = self.sub_schema_map.and_then(|sub_schema_map| {
            sub_schema_map
                .iter()
                .find_map(|(sub_root_accessors, sub_schema)| {
                    let mut sub_schema_uri = sub_schema.schema_uri.clone();
                    sub_schema_uri.set_fragment(None);
                    (sub_schema_uri == normalized
                        && sub_root_accessors.len() <= accessors.len()
                        && sub_root_accessors
                            .iter()
                            .zip(accessors.iter())
                            .all(|(expected, actual)| expected == actual))
                    .then_some(sub_root_accessors.as_slice())
                })
        });

        self.store
            .array_values_order_enabled_for_schema(&normalized, sub_root_accessors)
            .await
            .unwrap_or(true)
    }

    pub fn array_order_override(&self, accessors: &[Accessor]) -> Option<&ArrayOrderOverride> {
        self.overrides
            .array_values_order
            .and_then(|overrides| overrides.get(accessors))
    }

    pub async fn array_values_order(
        &self,
        accessors: &[Accessor],
        current_schema: Option<&CurrentSchema<'_>>,
        comment_directive_override: Option<&ArrayOrderOverride>,
    ) -> Option<XTombiArrayValuesOrder> {
        if let Some(comment_directive_override) = comment_directive_override {
            if comment_directive_override.disabled {
                return None;
            }
            if let Some(order) = comment_directive_override.order {
                return Some(XTombiArrayValuesOrder::All(order));
            }
        }

        if let Some(schema_override) = self.array_order_override(accessors) {
            if schema_override.disabled {
                return None;
            }
            if let Some(order) = schema_override.order {
                return Some(XTombiArrayValuesOrder::All(order));
            }
        }

        let current_schema = current_schema?;
        self.array_values_order_enabled(accessors, current_schema.schema_uri.as_ref())
            .await
            .then_some(())
            .and_then(|_| match current_schema.value_schema.as_ref() {
                ValueSchema::Array(array_schema) => array_schema.values_order.clone(),
                _ => None,
            })
    }

    pub async fn table_keys_order(
        &self,
        accessors: &[Accessor],
        current_schema: Option<&CurrentSchema<'_>>,
        comment_directive_override: Option<&TableOrderOverride>,
    ) -> Option<XTombiTableKeysOrder> {
        let (disabled, order) = comment_directive_override
            .map(|override_item| (override_item.disabled, override_item.order))
            .unwrap_or_default();
        if disabled {
            return None;
        }
        if let Some(order) = order {
            return Some(XTombiTableKeysOrder::All(order));
        }

        let (disabled, order) = self
            .table_order_override(accessors)
            .map(|override_item| (override_item.disabled, override_item.order))
            .unwrap_or_default();
        if disabled {
            return None;
        }
        if let Some(order) = order {
            return Some(XTombiTableKeysOrder::All(order));
        }

        let current_schema = current_schema?;
        self.table_keys_order_enabled(accessors, current_schema.schema_uri.as_ref())
            .await
            .then_some(())
            .and_then(|_| match current_schema.value_schema.as_ref() {
                ValueSchema::Table(table_schema) => table_schema.keys_order.clone(),
                _ => None,
            })
    }

    pub async fn get_subschema(
        &self,
        accessors: &[crate::Accessor],
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<Result<std::sync::Arc<crate::DocumentSchema>, crate::Error>> {
        if let Some(sub_schema_map) = self.sub_schema_map
            && let Some((_, sub_schema)) = sub_schema_map
                .iter()
                .find(|(pattern, _)| pattern.as_slice() == accessors)
            && current_schema
                .is_none_or(|current_schema| &*current_schema.schema_uri != &sub_schema.schema_uri)
        {
            return self
                .store
                .try_get_document_schema(&sub_schema.schema_uri)
                .await
                .transpose();
        }
        None
    }
}
