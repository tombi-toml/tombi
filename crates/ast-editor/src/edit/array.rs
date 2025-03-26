use std::borrow::Cow;

use futures::FutureExt;
use itertools::Itertools;
use schema_store::ValueSchema;

use crate::rule::{array_comma_tailing_comment, array_values_order};

impl crate::Edit for ast::Array {
    fn edit<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [schema_store::SchemaAccessor],
        current_schema: Option<&'a schema_store::CurrentSchema<'a>>,
        schema_context: &'a schema_store::SchemaContext<'a>,
    ) -> futures::future::BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];

            if let Some(current_schema) = current_schema {
                if let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref() {
                    changes.extend(
                        array_values_order(
                            self.values_with_comma().collect_vec(),
                            array_schema,
                            schema_context,
                        )
                        .await,
                    );

                    if let Some(item_schema) = &array_schema.items {
                        if let Ok(Some(current_schema)) = item_schema
                            .write()
                            .await
                            .resolve(
                                Cow::Borrowed(&current_schema.schema_url),
                                Cow::Borrowed(&current_schema.definitions),
                                schema_context.store,
                            )
                            .await
                        {
                            for value in self.values() {
                                changes.extend(
                                    value.edit(&[], Some(&current_schema), schema_context).await,
                                );
                            }

                            return changes;
                        }
                    }
                }
            }

            for (value, comma) in self.values_with_comma() {
                changes.extend(array_comma_tailing_comment(
                    &value,
                    comma.as_ref(),
                    schema_context,
                ));
                changes.extend(value.edit(&[], None, schema_context).await);
            }

            changes
        }
        .boxed()
    }
}
