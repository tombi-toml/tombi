use itertools::Itertools;
use tombi_ast::{AstNode, DanglingCommentGroupOr};
use tombi_comment_directive::value::{TableCommonFormatRules, TableCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;

use crate::rule::{inline_table_comma_trailing_comment, inline_table_keys_order};

impl crate::Edit for tombi_ast::InlineTable {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        log::trace!("node = {:?}", node);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("current_schema = {:?}", current_schema);

        async move {
            let mut changes = vec![];
            let total_key_values = self.key_values().count();
            let has_last_comma = !self.has_last_key_value_trailing_comma();
            let mut key_value_index = 0usize;

            for group in self.key_value_with_comma_groups() {
                let DanglingCommentGroupOr::ItemGroup(kv_group) = group else {
                    continue;
                };

                for (key_value, comma) in kv_group.key_values_with_comma() {
                    let is_last_key_value = key_value_index + 1 == total_key_values;
                    changes.extend(inline_table_comma_trailing_comment(
                        &key_value,
                        comma.as_ref(),
                        !has_last_comma || !is_last_key_value,
                    ));
                    changes.extend(
                        key_value
                            .edit(node, accessors, source_path, current_schema, schema_context)
                            .await,
                    );
                    key_value_index += 1;
                }
            }

            let comment_directive =
                get_comment_directive_content::<TableCommonFormatRules, TableCommonLintRules>(
                    if let Some(key_value) =
                        self.syntax().parent().and_then(tombi_ast::KeyValue::cast)
                    {
                        key_value
                            .comment_directives()
                            .chain(self.comment_directives())
                            .collect_vec()
                    } else {
                        self.comment_directives().collect_vec()
                    },
                );

            for group in self.key_value_with_comma_groups() {
                let DanglingCommentGroupOr::ItemGroup(key_value_group) = group else {
                    continue;
                };

                changes.extend(
                    inline_table_keys_order(
                        node,
                        key_value_group.key_values_with_comma().collect_vec(),
                        current_schema,
                        schema_context,
                        comment_directive.clone(),
                    )
                    .await,
                );
            }

            changes
        }
        .boxed()
    }
}
