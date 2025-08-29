use std::borrow::Cow;

use itertools::Itertools;
use tombi_ast::GetHeaderSchemarAccessors;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;

use crate::{edit::get_schema, rule::table_keys_order};

impl crate::Edit for tombi_ast::Table {
    fn edit<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        tracing::trace!("current_schema = {:?}", current_schema);

        async move {
            let mut changes = vec![];
            let Some(header_accessors) = self.get_header_accessor(schema_context.toml_version)
            else {
                return changes;
            };

            let mut value = &tombi_document_tree::Value::Table(
                self.clone()
                    .into_document_tree_and_errors(schema_context.toml_version)
                    .tree,
            );

            let current_schema = if let Some(current_schema) = current_schema {
                get_schema(value, &header_accessors, current_schema, schema_context)
                    .await
                    .map(|value_schema| tombi_schema_store::CurrentSchema {
                        value_schema: Cow::Owned(value_schema),
                        schema_uri: current_schema.schema_uri.clone(),
                        definitions: current_schema.definitions.clone(),
                    })
            } else {
                None
            };

            for header_accessor in &header_accessors {
                match (value, header_accessor) {
                    (tombi_document_tree::Value::Table(table), Accessor::Key(key)) => {
                        let Some(v) = table.get(key) else {
                            return changes;
                        };
                        value = v;
                    }
                    (tombi_document_tree::Value::Array(array), Accessor::Index(_)) => {
                        let Some(v) = array.get(0) else {
                            return changes;
                        };
                        value = v;
                    }
                    _ => {}
                }
            }

            for key_value in self.key_values() {
                changes.extend(
                    key_value
                        .edit(
                            &header_accessors,
                            source_path,
                            current_schema.as_ref(),
                            schema_context,
                        )
                        .await,
                );
            }

            changes.extend(
                table_keys_order(
                    value,
                    self.key_values().collect_vec(),
                    current_schema.as_ref(),
                    schema_context,
                )
                .await,
            );

            changes
        }
        .boxed()
    }
}
