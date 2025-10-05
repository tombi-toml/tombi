use itertools::Itertools;
use tombi_comment_directive::value::{TableCommonFormatRules, TableCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;
use tombi_syntax::SyntaxElement;

use crate::rule::root_table_keys_order;
use tombi_ast::AstToken;

impl crate::Edit for tombi_ast::Root {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        _accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];
            let mut key_values = vec![];
            let mut table_or_array_of_tables = vec![];

            // Move document schema/tombi comment directive to the top.
            if self
                .schema_document_comment_directive(source_path)
                .is_some()
                || !self.tombi_document_comment_directives().is_empty()
            {
                if let Some(document_header_comments) = self.get_document_header_comments() {
                    changes.push(crate::Change::AppendTop {
                        new: document_header_comments
                            .into_iter()
                            .map(|comment| SyntaxElement::Token(comment.syntax().clone()))
                            .collect_vec(),
                    });
                }
            }

            for key_value in self.key_values() {
                changes.extend(
                    key_value
                        .edit(node, &[], source_path, current_schema, schema_context)
                        .await,
                );
                key_values.push(key_value);
            }

            for table_or_array_of_table in self.table_or_array_of_tables() {
                match &table_or_array_of_table {
                    tombi_ast::TableOrArrayOfTable::Table(table) => {
                        changes.extend(
                            table
                                .edit(node, &[], source_path, current_schema, schema_context)
                                .await,
                        );
                    }
                    tombi_ast::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                        changes.extend(
                            array_of_table
                                .edit(node, &[], source_path, current_schema, schema_context)
                                .await,
                        );
                    }
                };
                table_or_array_of_tables.push(table_or_array_of_table);
            }

            let comment_directive = get_comment_directive_content::<
                TableCommonFormatRules,
                TableCommonLintRules,
            >(self.comment_directives());

            changes.extend(
                root_table_keys_order(
                    key_values,
                    table_or_array_of_tables,
                    current_schema,
                    schema_context,
                    comment_directive,
                )
                .await,
            );

            changes
        }
        .boxed()
    }
}
