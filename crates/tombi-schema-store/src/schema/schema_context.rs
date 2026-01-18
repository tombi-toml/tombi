use itertools::Itertools;

use super::{SchemaAccessor, SchemaUri};
use crate::Accessor;

pub struct SchemaContext<'a> {
    pub toml_version: tombi_config::TomlVersion,
    pub document_schema: Option<&'a crate::DocumentSchema>,
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
        let document_schema = self.document_schema?;
        let sub_schema_uri = {
            let sub_schema_uri_map = document_schema.sub_schema_uri_map.read().await;
            sub_schema_uri_map
                .get(&accessors.iter().map(SchemaAccessor::from).collect_vec())
                .cloned()
        };
        if let Some(sub_schema_uri) = sub_schema_uri {
            if current_schema
                .is_none_or(|current_schema| &*current_schema.schema_uri != &sub_schema_uri)
            {
                return self
                    .store
                    .try_get_document_schema(&sub_schema_uri)
                    .await
                    .transpose();
            }
        }
        None
    }

    pub async fn register_dynamic_sub_schema(
        &self,
        accessors: &[Accessor],
        schema_uri: &SchemaUri,
    ) {
        let Some(root_schema) = self.document_schema else {
            return;
        };

        if accessors.is_empty()
            || root_schema
                .schema_uri
                .as_ref()
                .is_some_and(|root_schema_uri| root_schema_uri == schema_uri)
        {
            return;
        }

        root_schema
            .register_dynamic_sub_schema(
                accessors.iter().map(SchemaAccessor::from).collect_vec(),
                schema_uri,
            )
            .await;
    }
}
