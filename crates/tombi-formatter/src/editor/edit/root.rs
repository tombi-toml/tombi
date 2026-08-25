use std::borrow::Cow;

use itertools::Itertools;
use tombi_ast_syntax::DocumentCommentDirectives;
use tombi_comment_directive::value::{TableCommonFormatRules, TableCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{Accessor, CurrentSchema, PatternAccessor};

use crate::editor::rule::root_table_keys_order::root_table_keys_order;
use tombi_ast_syntax::{DanglingCommentGroupOr, GetHeaderAccessors};
use tombi_schema_store::TableOrderOverride;

impl crate::editor::Edit for tombi_ast_syntax::Root {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree_syntax::Value,
        _accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::editor::Change>> {
        async move {
            let mut changes = vec![];
            let mut key_value_groups = vec![];
            let mut table_or_array_of_tables = vec![];
            let mut table_order_overrides = tombi_schema_store::TableOrderOverrides::default();

            // Detect document comment directives.
            // If dangling comments exist, directives should already be there.
            // Otherwise, check first item's leading comments and move them to the top.
            let document_comment_directives = if self.dangling_comment_groups().next().is_some() {
                DocumentCommentDirectives::from_comments(
                    self.dangling_comment_groups()
                        .flat_map(|comment_group| comment_group.into_comments().map(Into::into)),
                    source_path,
                )
            } else {
                DocumentCommentDirectives::from_comments(
                    self.first_item_leading_comments().map(Into::into),
                    source_path,
                )
                .inspect(|_| {
                    let comments = self
                        .first_item_leading_comments()
                        .map(|comment| comment.syntax().text().to_string())
                        .join("\n");
                    if !comments.is_empty() {
                        changes.push(crate::editor::Change::append_top(format!("{comments}\n\n")));
                        for comment in self.first_item_leading_comments() {
                            changes.push(crate::editor::Change::remove_token(comment.syntax()));
                        }
                    }
                })
            };

            let current_schema_from_directive =
                if let Some(ref document_comment_directives) = document_comment_directives {
                    resolve_current_schema_from_comment_directive(
                        document_comment_directives,
                        current_schema,
                        schema_context,
                    )
                    .await
                } else {
                    None
                };

            let current_schema = current_schema_from_directive.as_ref().or(current_schema);

            for group in self.key_value_groups() {
                let DanglingCommentGroupOr::ItemGroup(key_value_group) = group else {
                    continue;
                };

                for key_value in key_value_group.key_values() {
                    changes.extend(
                        key_value
                            .edit(node, &[], source_path, current_schema, schema_context)
                            .await,
                    );
                }
                key_value_groups.push(key_value_group);
            }

            for table_or_array_of_table in self.table_or_array_of_tables() {
                match &table_or_array_of_table {
                    tombi_ast_syntax::TableOrArrayOfTable::Table(table) => {
                        changes.extend(
                            table
                                .edit(node, &[], source_path, current_schema, schema_context)
                                .await,
                        );
                    }
                    tombi_ast_syntax::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                        changes.extend(
                            array_of_table
                                .edit(node, &[], source_path, current_schema, schema_context)
                                .await,
                        );
                    }
                };

                if let Some(header_accessors) =
                    table_or_array_of_table.get_header_accessors(schema_context.toml_version)
                {
                    let comment_directive = match &table_or_array_of_table {
                        tombi_ast_syntax::TableOrArrayOfTable::Table(table) => {
                            get_comment_directive_content::<
                                TableCommonFormatRules,
                                TableCommonLintRules,
                            >(table.comment_directives())
                        }
                        tombi_ast_syntax::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                            get_comment_directive_content::<
                                TableCommonFormatRules,
                                TableCommonLintRules,
                            >(array_of_table.comment_directives())
                        }
                    };

                    if let Some(comment_directive) = comment_directive {
                        let disabled = comment_directive
                            .table_keys_order_disabled()
                            .unwrap_or_default();
                        let order = comment_directive.table_keys_order().map(Into::into);
                        if disabled || order.is_some() {
                            table_order_overrides.push(TableOrderOverride {
                                target: accessors_to_root(&header_accessors),
                                disabled,
                                order,
                            });
                        }
                    }
                }

                table_or_array_of_tables.push(table_or_array_of_table);
            }

            let comment_directive = get_comment_directive_content::<
                TableCommonFormatRules,
                TableCommonLintRules,
            >(self.comment_directives());

            changes.extend(
                root_table_keys_order(
                    key_value_groups,
                    table_or_array_of_tables,
                    current_schema,
                    schema_context,
                    comment_directive,
                    Some(&table_order_overrides),
                )
                .await,
            );

            changes
        }
        .boxed()
    }
}

fn accessors_to_root(accessors: &[Accessor]) -> Vec<PatternAccessor> {
    accessors
        .iter()
        .map(|accessor| match accessor {
            Accessor::Key(key) => PatternAccessor::Key(key.clone()),
            Accessor::Index(index) => PatternAccessor::Index(*index),
        })
        .collect()
}

async fn resolve_current_schema_from_comment_directive<'a>(
    document_comment_directives: &DocumentCommentDirectives,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> Option<CurrentSchema<'a>> {
    if current_schema.is_some() {
        return None;
    }
    let schema_directive = document_comment_directives.schema.as_ref()?;
    let schema_uri = schema_directive.uri.as_ref().ok()?;
    let document_schema = schema_context
        .store
        .try_get_document_schema(schema_uri)
        .await
        .ok()??;

    Some(CurrentSchema {
        schema_view: document_schema.schema_view.clone()?,
        semantic_schema: document_schema.semantic_schema.clone(),
        schema_uri: Cow::Owned(document_schema.schema_uri.clone()),
        definitions: Cow::Owned(document_schema.definitions.clone()),
        strict: document_schema.strict,
    })
}
