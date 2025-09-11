use std::borrow::Cow;

use itertools::Itertools;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;

use crate::rule::{array_comma_trailing_comment, array_values_order};

impl crate::Edit for tombi_ast::Array {
    fn edit<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];

            for (value, comma) in self.values_with_comma() {
                changes.extend(array_comma_trailing_comment(
                    &value,
                    comma.as_ref(),
                    schema_context,
                ));
                changes.extend(value.edit(&[], source_path, None, schema_context).await);
            }

            if let Some(current_schema) = current_schema {
                tracing::debug!("Current schema: {:?}", current_schema.value_schema);
                if let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref() {
                    tracing::debug!("Array schema values_order: {:?}", array_schema.values_order);
                    if let Some(values_order) = &array_schema.values_order {
                        tracing::debug!("Processing array values order: {:?}", values_order);
                        if let Ok(array_value) = self
                            .clone()
                            .try_into_document_tree(schema_context.toml_version)
                        {
                            changes.extend(
                                array_values_order(
                                    &array_value,
                                    self.values_with_comma().collect_vec(),
                                    array_schema,
                                    &current_schema,
                                    schema_context,
                                )
                                .await,
                            );
                        }
                    } else {
                        tracing::debug!("No values_order found in array schema");
                    }
                } else {
                    tracing::debug!("Not an array schema: {:?}", current_schema.value_schema);
                }

                if let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref() {
                    if let Some(item_schema) = &array_schema.items {
                        if let Ok(Some(current_schema)) = item_schema
                            .write()
                            .await
                            .resolve(
                                Cow::Borrowed(&current_schema.schema_uri),
                                Cow::Borrowed(&current_schema.definitions),
                                schema_context.store,
                            )
                            .await
                            .inspect_err(|err| tracing::warn!("{err}"))
                        {
                            for value in self.values() {
                                changes.extend(
                                    value
                                        .edit(
                                            &[],
                                            source_path,
                                            Some(&current_schema),
                                            schema_context,
                                        )
                                        .await,
                                );
                            }
                        }
                    }
                }
            }

            changes
        }
        .boxed()
    }
}
