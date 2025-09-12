use std::borrow::Cow;

use itertools::Itertools;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{Accessor, CurrentSchema};

use super::get_schema;

impl crate::Edit for tombi_ast::KeyValue {
    fn edit<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];

            let Some(keys) = self.keys() else {
                return changes;
            };

            let keys_accessors = keys
                .keys()
                .map(|key| Accessor::Key(key.to_raw_text(schema_context.toml_version)))
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
                                    source_path,
                                    Some(&CurrentSchema {
                                        value_schema: Cow::Owned(value_schema),
                                        schema_uri: current_schema.schema_uri.clone(),
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
                changes.extend(value.edit(&[], source_path, None, schema_context).await);
            }

            changes
        }
        .boxed()
    }
}
