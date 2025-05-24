use std::borrow::Cow;

use tombi_document_tree::TryIntoDocumentTree;
use futures::FutureExt;
use itertools::Itertools;
use tombi_schema_store::{CurrentSchema, SchemaAccessor};

use super::get_schema;

impl crate::Edit for tombi_ast::KeyValue {
    fn edit<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> futures::future::BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];

            let Some(keys) = self.keys() else {
                return changes;
            };

            let keys_accessors = keys
                .keys()
                .filter_map(|key| {
                    key.try_to_raw_text(schema_context.toml_version)
                        .ok()
                        .map(SchemaAccessor::Key)
                })
                .collect_vec();

            if let Some(current_schema) = current_schema {
                if let Some(value_schema) = get_schema(
                    &tombi_document_tree::Value::Table(
                        self.clone()
                            .try_into_document_tree(schema_context.toml_version)
                            .unwrap(),
                    ),
                    &keys_accessors.clone(),
                    current_schema,
                    schema_context,
                )
                .await
                {
                    if let Some(value) = self.value() {
                        changes.extend(
                            value
                                .edit(
                                    &[],
                                    Some(&CurrentSchema {
                                        value_schema: Cow::Owned(value_schema),
                                        schema_url: current_schema.schema_url.clone(),
                                        definitions: current_schema.definitions.clone(),
                                    }),
                                    schema_context,
                                )
                                .await,
                        );
                        return changes;
                    }
                }
            }

            if let Some(value) = self.value() {
                changes.extend(value.edit(&[], None, schema_context).await);
            }

            changes
        }
        .boxed()
    }
}
