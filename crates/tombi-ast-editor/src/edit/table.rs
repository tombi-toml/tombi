use std::sync::Arc;

use itertools::Itertools;
use tombi_ast::GetHeaderAccessors;
use tombi_comment_directive::value::{TableCommonFormatRules, TableCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;

use crate::{edit::edit_recursive, rule::table_keys_order};

impl crate::Edit for tombi_ast::Table {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        tracing::trace!("current_schema = {:?}", current_schema);

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
                        tracing::trace!("node = {:#?}", node);
                        tracing::trace!("accessors = {:?}", accessors);
                        tracing::trace!("current_schema = {:#?}", current_schema);

                        let mut changes = vec![];
                        for key_value in self.key_values() {
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
                            )
                        }
                        changes.extend(
                            table_keys_order(
                                node,
                                self.key_values().collect_vec(),
                                current_schema.as_ref(),
                                schema_context,
                                comment_directive,
                            )
                            .await,
                        );

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
