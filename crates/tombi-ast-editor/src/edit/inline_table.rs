use crate::compat::{BoxFuture, Boxable};
use itertools::Itertools;
use tombi_schema_store::ValueSchema;

use crate::rule::{inline_table_comma_tailing_comment, inline_table_keys_order};

impl crate::Edit for tombi_ast::InlineTable {
    fn edit<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];

            if let Some(current_schema) = current_schema {
                if let ValueSchema::Table(table_schema) = current_schema.value_schema.as_ref() {
                    changes.extend(
                        inline_table_keys_order(
                            self.key_values_with_comma().collect_vec(),
                            table_schema,
                            schema_context,
                        )
                        .await,
                    );

                    for key_value in self.key_values() {
                        changes.extend(
                            key_value
                                .edit(accessors, Some(current_schema), schema_context)
                                .await,
                        );
                    }

                    return changes;
                }
            }

            for (key_value, comma) in self.key_values_with_comma() {
                changes.extend(inline_table_comma_tailing_comment(
                    &key_value,
                    comma.as_ref(),
                ));
                changes.extend(key_value.edit(accessors, None, schema_context).await);
            }

            changes
        }
        .boxed()
    }
}
