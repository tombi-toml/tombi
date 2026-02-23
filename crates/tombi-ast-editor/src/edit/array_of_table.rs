use std::sync::Arc;

use itertools::Itertools;
use tombi_ast::{DanglingCommentGroupOr, GetHeaderAccessors};
use tombi_comment_directive::value::{TableCommonFormatRules, TableCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;

use crate::{edit::edit_recursive, rule::table_keys_order};

impl crate::Edit for tombi_ast::ArrayOfTable {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        log::trace!("current_schema = {:?}", current_schema);

        async move {
            let Some(header_accessors) = self.get_header_accessors(schema_context.toml_version)
            else {
                return Vec::with_capacity(0);
            };

            let comment_directive = get_comment_directive_content::<
                TableCommonFormatRules,
                TableCommonLintRules,
            >(self.comment_directives());

            edit_recursive(
                node,
                |node, accessors, current_schema| {
                    async move {
                        log::trace!("node = {:?}", node);
                        log::trace!("accessors = {:?}", accessors);
                        log::trace!("current_schema = {:?}", current_schema);

                        let mut changes = vec![];
                        for group in self.key_value_groups() {
                            let DanglingCommentGroupOr::ItemGroup(key_value_group) = group else {
                                continue;
                            };

                            let key_values = key_value_group.key_values().collect_vec();
                            for key_value in &key_values {
                                changes.extend(
                                    key_value
                                        .edit(
                                            node,
                                            &accessors,
                                            source_path,
                                            current_schema.as_ref(),
                                            schema_context,
                                        )
                                        .await,
                                );
                            }

                            changes.extend(
                                table_keys_order(
                                    node,
                                    key_values,
                                    current_schema.as_ref(),
                                    schema_context,
                                    comment_directive.clone(),
                                )
                                .await,
                            );
                        }

                        changes
                    }
                    .boxed()
                },
                &header_accessors,
                Arc::from(accessors.to_vec()),
                current_schema.cloned(),
                schema_context,
            )
            .await
        }
        .boxed()
    }
}
