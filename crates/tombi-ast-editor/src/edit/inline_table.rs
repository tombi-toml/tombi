use itertools::Itertools;
use tombi_comment_directive::value::{TableCommonLintRules, TableFormatRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_future::{BoxFuture, Boxable};

use crate::rule::{inline_table_comma_trailing_comment, table_keys_order};

impl crate::Edit for tombi_ast::InlineTable {
    fn edit<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let mut changes = vec![];

            let comment_directive = get_comment_directive_content::<
                TableFormatRules,
                TableCommonLintRules,
            >(self.comment_directives());

            let value = &self
                .clone()
                .into_document_tree_and_errors(schema_context.toml_version)
                .tree;

            for (key_value, comma) in self.key_values_with_comma() {
                changes.extend(inline_table_comma_trailing_comment(
                    &key_value,
                    comma.as_ref(),
                ));
                changes.extend(
                    key_value
                        .edit(accessors, source_path, current_schema, schema_context)
                        .await,
                );
            }

            if let Some(current_schema) = current_schema {
                changes.extend(
                    table_keys_order(
                        value,
                        self.key_values().collect_vec(),
                        Some(current_schema),
                        schema_context,
                        comment_directive,
                    )
                    .await,
                );
            }

            changes
        }
        .boxed()
    }
}
