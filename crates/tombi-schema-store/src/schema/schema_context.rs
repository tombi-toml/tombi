use itertools::Itertools;
use tombi_severity_level::SeverityLevelDefaultWarn;
use tombi_x_keyword::StringFormat;

use crate::schema::schema_cycle_guard::SchemaVisits;

use super::SchemaAccessor;

pub struct SchemaContext<'a> {
    pub toml_version: tombi_config::TomlVersion,
    pub root_schema: Option<&'a crate::DocumentSchema>,
    pub sub_schema_uri_map: Option<&'a crate::SubSchemaUriMap>,
    pub deprecated_lint_level: Option<SeverityLevelDefaultWarn>,
    pub schema_visits: SchemaVisits,
    pub store: &'a crate::SchemaStore,
    pub strict: Option<bool>,
}

impl SchemaContext<'_> {
    #[inline]
    pub fn strict(&self) -> bool {
        self.strict.unwrap_or_else(|| self.store.strict())
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

    pub async fn get_subschema(
        &self,
        accessors: &[crate::Accessor],
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<Result<std::sync::Arc<crate::DocumentSchema>, crate::Error>> {
        if let Some(sub_schema_uri_map) = self.sub_schema_uri_map
            && let Some(sub_schema_uri) =
                sub_schema_uri_map.get(&accessors.iter().map(SchemaAccessor::from).collect_vec())
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
