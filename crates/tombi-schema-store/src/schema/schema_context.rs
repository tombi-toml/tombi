use super::SchemaAccessor;

pub struct SchemaContext<'a> {
    pub toml_version: tombi_config::TomlVersion,
    pub root_schema: Option<&'a crate::DocumentSchema>,
    pub sub_schema_url_map: Option<&'a crate::SubSchemaUrlMap>,
    pub store: &'a crate::SchemaStore,
}

impl SchemaContext<'_> {
    #[inline]
    pub fn strict(&self) -> bool {
        self.store.strict()
    }

    pub async fn get_subschema(
        &self,
        accessors: &[crate::Accessor],
        current_schema: Option<&crate::CurrentSchema<'_>>,
    ) -> Option<Result<crate::DocumentSchema, crate::Error>> {
        if let Some(sub_schema_url_map) = self.sub_schema_url_map {
            if let Some(sub_schema_url) = sub_schema_url_map.get(
                &accessors
                    .iter()
                    .map(SchemaAccessor::from)
                    .collect::<Vec<_>>(),
            ) {
                if current_schema
                    .is_none_or(|current_schema| &*current_schema.schema_url != sub_schema_url)
                {
                    return self
                        .store
                        .try_get_document_schema(sub_schema_url)
                        .await
                        .transpose();
                }
            }
        }
        None
    }
}
