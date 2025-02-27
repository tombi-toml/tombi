use std::borrow::Cow;

use ast::AstNode;
use futures::FutureExt;
use schema_store::{CurrentSchema, ValueSchema};

use crate::rule::array_values_order_by;

impl crate::Edit for ast::Array {
    fn edit<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [schema_store::Accessor],
        schema_url: Option<&'a schema_store::SchemaUrl>,
        value_schema: Option<&'a schema_store::ValueSchema>,
        definitions: Option<&'a schema_store::SchemaDefinitions>,
        schema_context: &'a schema_store::SchemaContext<'a>,
    ) -> futures::future::BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];

            if let (Some(schema_url), Some(ValueSchema::Array(array_schema)), Some(definitions)) =
                (schema_url, value_schema, definitions)
            {
                if let Some(item_schema) = &array_schema.items {
                    if let Ok(CurrentSchema {
                        schema_url,
                        value_schema,
                        definitions,
                    }) = item_schema
                        .write()
                        .await
                        .resolve(
                            Cow::Borrowed(schema_url),
                            Cow::Borrowed(definitions),
                            &schema_context.store,
                        )
                        .await
                    {
                        for value in self.values() {
                            changes.extend(
                                value
                                    .edit(
                                        accessors,
                                        Some(&schema_url),
                                        Some(value_schema),
                                        Some(&definitions),
                                        schema_context,
                                    )
                                    .await,
                            );
                        }
                    }
                }
                changes.extend(
                    array_values_order_by(self.syntax(), array_schema, schema_context.toml_version)
                        .await,
                );
            } else {
                for value in self.values() {
                    changes.extend(
                        value
                            .edit(accessors, None, None, None, schema_context)
                            .await,
                    );
                }
            }

            changes
        }
        .boxed()
    }
}
