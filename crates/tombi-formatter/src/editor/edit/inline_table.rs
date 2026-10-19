use itertools::Itertools;
use tombi_ast_syntax::DanglingCommentGroupOr;
use tombi_comment_directive::value::{TableCommonFormatRules, TableCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;

use crate::editor::rule::{
    inline_table_comma_trailing_comment::inline_table_comma_trailing_comment,
    inline_table_keys_order::inline_table_keys_order,
};

impl<'t> crate::editor::Edit for tombi_ast_syntax::InlineTable<'t> {
    fn edit<'a: 'b, 'b, 'd>(
        &'a self,
        node: &'a tombi_document_tree_syntax::Value<'d>,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::editor::Change>> {
        log::trace!("node = {:?}", node);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("current_schema = {:?}", current_schema);

        async move {
            let mut changes = vec![];
            let key_value_groups = self
                .key_value_with_comma_groups()
                .filter_map(DanglingCommentGroupOr::into_item_group)
                .map(|group| group.key_values_with_comma().collect_vec())
                .collect_vec();
            let total_key_values: usize = key_value_groups.iter().map(Vec::len).sum();
            let last_key_value_has_comma = key_value_groups
                .iter()
                .rev()
                .find_map(|group| group.last())
                .is_some_and(|(_, comma)| comma.is_some());
            let mut key_value_index = 0usize;

            for key_value_group in &key_value_groups {
                for (key_value, comma) in key_value_group {
                    let is_last_key_value = key_value_index + 1 == total_key_values;
                    changes.extend(inline_table_comma_trailing_comment(
                        key_value,
                        comma.as_ref(),
                        last_key_value_has_comma || !is_last_key_value,
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
                    if let Some(key_value) = self.parent_key_value() {
                        key_value
                            .comment_directives()
                            .chain(self.comment_directives())
                            .collect_vec()
                    } else {
                        self.comment_directives().collect_vec()
                    },
                );

            for key_value_group in key_value_groups {
                changes.extend(
                    inline_table_keys_order(
                        node,
                        accessors,
                        key_value_group,
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
