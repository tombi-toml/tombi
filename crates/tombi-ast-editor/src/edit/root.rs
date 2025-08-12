use itertools::Itertools;
use tombi_future::{BoxFuture, Boxable};
use tombi_syntax::SyntaxElement;

use crate::rule::root_table_keys_order;
use tombi_ast::AstToken;

impl crate::Edit for tombi_ast::Root {
    fn edit<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];
            let mut key_values = vec![];
            let mut table_or_array_of_tables = vec![];

            if self.schema_comment_directive(None).is_some() {
                changes.push(crate::Change::AppendTop {
                    new: self
                        .get_document_header_comments()
                        .unwrap()
                        .into_iter()
                        .map(|comment| SyntaxElement::Token(comment.syntax().clone()))
                        .collect_vec(),
                });
            }

            for key_value in self.key_values() {
                changes.extend(key_value.edit(&[], current_schema, schema_context).await);
                key_values.push(key_value);
            }

            for table_or_array_of_table in self.table_or_array_of_tables() {
                match &table_or_array_of_table {
                    tombi_ast::TableOrArrayOfTable::Table(table) => {
                        changes.extend(table.edit(&[], current_schema, schema_context).await);
                    }
                    tombi_ast::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                        changes.extend(
                            array_of_table
                                .edit(&[], current_schema, schema_context)
                                .await,
                        );
                    }
                };
                table_or_array_of_tables.push(table_or_array_of_table);
            }

            changes.extend(
                root_table_keys_order(
                    key_values,
                    table_or_array_of_tables,
                    current_schema,
                    schema_context,
                )
                .await,
            );

            changes
        }
        .boxed()
    }
}
