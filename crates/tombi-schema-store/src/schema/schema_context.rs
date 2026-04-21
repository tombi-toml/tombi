use tombi_config::{SchemaFormatRules, SchemaLintRules};
use tombi_severity_level::SeverityLevelDefaultWarn;
use tombi_x_keyword::StringFormat;

use crate::schema::schema_cycle_guard::SchemaVisits;

pub struct SchemaContext<'a> {
    pub toml_version: tombi_config::TomlVersion,
    pub root_schema: Option<&'a crate::DocumentSchema>,
    pub sub_schema_uri_map: Option<&'a crate::SubSchemaUriMap>,
    pub deprecated_lint_level: Option<SeverityLevelDefaultWarn>,
    pub schema_format_rules: Option<&'a crate::SchemaFormatRulesMap>,
    pub schema_lint_rules: Option<&'a crate::SchemaLintRulesMap>,
    pub schema_overrides: Option<&'a crate::SchemaOverridesMap>,
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
            sub_schema_uri_map: source_schema.map(|schema| &schema.sub_schema_uri_map),
            deprecated_lint_level: source_schema.and_then(|schema| schema.deprecated_lint_level),
            schema_format_rules: source_schema.map(|schema| &schema.schema_format_rules),
            schema_lint_rules: source_schema.map(|schema| &schema.schema_lint_rules),
            schema_overrides: source_schema.map(|schema| &schema.schema_overrides),
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
            sub_schema_uri_map: self.sub_schema_uri_map,
            deprecated_lint_level: self.deprecated_lint_level,
            schema_format_rules: self.schema_format_rules,
            schema_lint_rules: self.schema_lint_rules,
            schema_overrides: self.schema_overrides,
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
    pub fn deprecated_lint_level(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
        accessors: &[crate::Accessor],
    ) -> Option<SeverityLevelDefaultWarn> {
        self.schema_overrides(current_schema)
            .and_then(|overrides| overrides.deprecated.find(accessors))
            .map(|override_item| override_item.level)
            .or_else(|| {
                self.schema_lint_rules(current_schema)
                    .and_then(|rules| rules.deprecated)
            })
            .or(self.deprecated_lint_level)
    }

    #[inline]
    pub fn schema_array_values_order_enabled(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> bool {
        self.schema_format_rules(current_schema)
            .and_then(|rules| rules.array_values_order.as_ref())
            .and_then(|rule| rule.enabled)
            .unwrap_or_default()
            .value()
    }

    #[inline]
    pub fn schema_table_keys_order_enabled(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> bool {
        self.schema_format_rules(current_schema)
            .and_then(|rules| rules.table_keys_order.as_ref())
            .and_then(|rule| rule.enabled)
            .unwrap_or_default()
            .value()
    }

    fn schema_format_rules(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<&SchemaFormatRules> {
        let schema_uri = self.normalize_schema_uri(current_schema)?;
        self.schema_format_rules
            .and_then(|rules| rules.get(&schema_uri))
            .or_else(|| self.root_schema_format_rules())
    }

    fn schema_lint_rules(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<&SchemaLintRules> {
        let schema_uri = self.normalize_schema_uri(current_schema)?;
        self.schema_lint_rules
            .and_then(|rules| rules.get(&schema_uri))
            .or_else(|| self.root_schema_lint_rules())
    }

    pub fn array_order_override(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
        accessors: &[crate::Accessor],
    ) -> Option<&crate::ArrayOrderOverride> {
        self.schema_overrides(current_schema)
            .and_then(|overrides| overrides.array_values_order.find(accessors))
            .or_else(|| {
                self.root_schema_overrides()
                    .and_then(|overrides| overrides.array_values_order.find(accessors))
            })
    }

    pub fn table_order_override(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
        accessors: &[crate::Accessor],
    ) -> Option<&crate::TableOrderOverride> {
        self.schema_overrides(current_schema)
            .and_then(|overrides| overrides.table_keys_order.find(accessors))
            .or_else(|| {
                self.root_schema_overrides()
                    .and_then(|overrides| overrides.table_keys_order.find(accessors))
            })
    }

    pub fn root_table_order_overrides(&self) -> Option<&crate::TableOrderOverrides> {
        Some(&self.root_schema_overrides()?.table_keys_order)
    }

    pub fn table_keys_order(
        &self,
        accessors: &[crate::Accessor],
        current_schema: Option<&crate::CurrentSchema<'_>>,
        comment_directive_override: Option<&crate::TableOrderOverride>,
    ) -> Option<crate::XTombiTableKeysOrder> {
        if let Some(override_item) = comment_directive_override {
            if override_item.disabled {
                return None;
            }
            if let Some(order) = override_item.order {
                return Some(crate::XTombiTableKeysOrder::All(order));
            }
        }

        if let Some(override_item) = self.table_order_override(current_schema, accessors) {
            if override_item.disabled {
                return None;
            }
            if let Some(order) = override_item.order {
                return Some(crate::XTombiTableKeysOrder::All(order));
            }
        }

        let current_schema = current_schema?;
        if !self.schema_table_keys_order_enabled(Some(current_schema)) {
            return None;
        }

        match current_schema.value_schema.as_ref() {
            crate::ValueSchema::Table(table_schema) => table_schema.keys_order.clone(),
            _ => None,
        }
    }

    pub fn array_values_order(
        &self,
        accessors: &[crate::Accessor],
        current_schema: Option<&crate::CurrentSchema<'_>>,
        comment_directive_override: Option<&crate::ArrayOrderOverride>,
    ) -> Option<crate::XTombiArrayValuesOrder> {
        if let Some(override_item) = comment_directive_override {
            if override_item.disabled {
                return None;
            }
            if let Some(order) = override_item.order {
                return Some(crate::XTombiArrayValuesOrder::All(order));
            }
        }

        if let Some(override_item) = self.array_order_override(current_schema, accessors) {
            if override_item.disabled {
                return None;
            }
            if let Some(order) = override_item.order {
                return Some(crate::XTombiArrayValuesOrder::All(order));
            }
        }

        let current_schema = current_schema?;
        if !self.schema_array_values_order_enabled(Some(current_schema)) {
            return None;
        }

        match current_schema.value_schema.as_ref() {
            crate::ValueSchema::Array(array_schema) => array_schema.values_order.clone(),
            _ => None,
        }
    }

    fn schema_overrides(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<&crate::SchemaOverrides> {
        let schema_uri = self.normalize_schema_uri(current_schema)?;
        self.schema_overrides?.get(&schema_uri)
    }

    fn root_schema_overrides(&self) -> Option<&crate::SchemaOverrides> {
        let document_schema = self.root_schema?;
        let mut schema_uri = document_schema.schema_uri.clone();
        if schema_uri.fragment().is_some() {
            schema_uri.set_fragment(None);
        }
        self.schema_overrides?.get(&schema_uri)
    }

    fn root_schema_format_rules(&self) -> Option<&SchemaFormatRules> {
        let document_schema = self.root_schema?;
        let mut schema_uri = document_schema.schema_uri.clone();
        if schema_uri.fragment().is_some() {
            schema_uri.set_fragment(None);
        }
        self.schema_format_rules?.get(&schema_uri)
    }

    fn root_schema_lint_rules(&self) -> Option<&SchemaLintRules> {
        let document_schema = self.root_schema?;
        let mut schema_uri = document_schema.schema_uri.clone();
        if schema_uri.fragment().is_some() {
            schema_uri.set_fragment(None);
        }
        self.schema_lint_rules?.get(&schema_uri)
    }

    fn normalize_schema_uri(
        &self,
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<crate::SchemaUri> {
        let current_schema = current_schema?;
        let mut schema_uri = current_schema.schema_uri.clone().into_owned();
        if schema_uri.fragment().is_some() {
            schema_uri.set_fragment(None);
        }
        Some(schema_uri)
    }

    pub async fn get_subschema(
        &self,
        accessors: &[crate::Accessor],
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<Result<std::sync::Arc<crate::DocumentSchema>, crate::Error>> {
        if let Some(sub_schema_uri_map) = self.sub_schema_uri_map
            && let Some((_, sub_schema_uri)) = sub_schema_uri_map
                .iter()
                .find(|(pattern, _)| pattern.as_slice() == accessors)
            && current_schema
                .is_none_or(|current_schema| &*current_schema.schema_uri != sub_schema_uri)
        {
            return self
                .store
                .try_get_document_schema(sub_schema_uri)
                .await
                .transpose();
        }
        None
    }
}
