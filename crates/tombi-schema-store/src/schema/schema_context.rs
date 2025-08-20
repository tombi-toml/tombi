use itertools::Itertools;

use super::SchemaAccessor;

pub struct SchemaContext<'a> {
    pub toml_version: tombi_config::TomlVersion,
    pub root_schema: Option<&'a crate::DocumentSchema>,
    pub sub_schema_uri_map: Option<&'a crate::SubSchemaUriMap>,
    pub store: &'a crate::SchemaStore,
    pub strict: Option<bool>,
}

impl SchemaContext<'_> {
    #[inline]
    pub fn strict(&self) -> bool {
        self.strict.unwrap_or_else(|| self.store.strict())
    }

    pub async fn get_subschema(
        &self,
        accessors: &[crate::Accessor],
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<Result<crate::DocumentSchema, crate::Error>> {
        if let Some(sub_schema_uri_map) = self.sub_schema_uri_map {
            if let Some(sub_schema_uri) =
                sub_schema_uri_map.get(&accessors.iter().map(SchemaAccessor::from).collect_vec())
            {
                if current_schema
                    .is_none_or(|current_schema| &*current_schema.schema_uri != sub_schema_uri)
                {
                    return self
                        .store
                        .try_get_document_schema(sub_schema_uri)
                        .await
                        .transpose();
                }
            }
        }
        None
    }
}
