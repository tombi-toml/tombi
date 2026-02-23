use std::borrow::Cow;

use itertools::Itertools;
use tombi_comment_directive::value::{TableCommonFormatRules, TableCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{Accessor, CurrentSchema};
use tombi_syntax::SyntaxElement;

use crate::node::make_dangling_comment_group_from_leading_comments;
use crate::rule::root_table_keys_order;
use crate::rule::{TableOrderOverride, TableOrderOverrides};
use tombi_ast::{
    ArrayOfTable, AstNode, DanglingCommentGroupOr, GetHeaderAccessors, KeyValueGroup, Table,
};

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
            let mut key_value_groups = vec![];
            let mut table_or_array_of_tables = vec![];
            let mut table_order_overrides = TableOrderOverrides::default();
            let has_root_document_comment_directive =
                has_root_document_comment_directive(self, source_path);
            let mut current_schema_from_first_leading_comments: Option<CurrentSchema<'a>> = None;

            if current_schema.is_none() && !has_root_document_comment_directive {
                current_schema_from_first_leading_comments =
                    resolve_current_schema_from_first_item_document_comment_directive(
                        self,
                        source_path,
                        schema_context,
                    )
                    .await;
            }
            let current_schema = current_schema_from_first_leading_comments
                .as_ref()
                .or(current_schema);

            // Move document schema/tombi comment directive to the top.
            if !has_root_document_comment_directive
                && let Some(first_leading_comments) =
                    first_item_document_leading_comments(self, source_path)
                && let Some(dangling_comment_group) =
                    make_dangling_comment_group_from_leading_comments(&first_leading_comments)
            {
                changes.push(crate::Change::AppendTop {
                    new: vec![SyntaxElement::Node(dangling_comment_group)],
                });
                for comment in first_leading_comments {
                    changes.push(crate::Change::Remove {
                        target: SyntaxElement::Token(comment.syntax().clone()),
                    });
                }
            }

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

                if let Some(header_accessors) =
                    table_or_array_of_table.get_header_accessors(schema_context.toml_version)
                {
                    let comment_directive = match &table_or_array_of_table {
                        tombi_ast::TableOrArrayOfTable::Table(table) => {
                            get_comment_directive_content::<
                                TableCommonFormatRules,
                                TableCommonLintRules,
                            >(table.comment_directives())
                        }
                        tombi_ast::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                            get_comment_directive_content::<
                                TableCommonFormatRules,
                                TableCommonLintRules,
                            >(array_of_table.comment_directives())
                        }
                    };

                    if let Some(comment_directive) = comment_directive {
                        let disabled = comment_directive
                            .table_keys_order_disabled()
                            .unwrap_or(false);
                        let order = comment_directive.table_keys_order().map(Into::into);
                        if disabled || order.is_some() {
                            table_order_overrides
                                .insert(header_accessors, TableOrderOverride { disabled, order });
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

fn has_root_document_comment_directive(
    root: &tombi_ast::Root,
    source_path: Option<&std::path::Path>,
) -> bool {
    root.dangling_comment_groups().any(|comment_group| {
        comment_group.comments().any(|comment| {
            comment.get_document_schema_directive(source_path).is_some()
                || comment.get_tombi_document_directive().is_some()
        })
    })
}

fn first_item_document_leading_comments(
    root: &tombi_ast::Root,
    source_path: Option<&std::path::Path>,
) -> Option<Vec<tombi_ast::LeadingComment>> {
    let comments = root.syntax().children().find_map(|node| {
        if let Some(key_value_group) = KeyValueGroup::cast(node.clone()) {
            Some(key_value_group.leading_comments().collect_vec())
        } else if let Some(table) = Table::cast(node.clone()) {
            Some(table.header_leading_comments().collect_vec())
        } else {
            ArrayOfTable::cast(node)
                .map(|array_of_table| array_of_table.header_leading_comments().collect_vec())
        }
    })?;

    comments
        .iter()
        .any(|comment| {
            comment.get_document_schema_directive(source_path).is_some()
                || comment.get_tombi_document_directive().is_some()
        })
        .then_some(comments)
}

async fn resolve_current_schema_from_first_item_document_comment_directive<'a>(
    root: &tombi_ast::Root,
    source_path: Option<&'a std::path::Path>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> Option<CurrentSchema<'a>> {
    let leading_comments = first_item_document_leading_comments(root, source_path)?;

    for comment in leading_comments {
        let Some(schema_directive) = comment.get_document_schema_directive(source_path) else {
            continue;
        };
        let Ok(schema_uri) = schema_directive.uri else {
            continue;
        };
        let Ok(Some(document_schema)) = schema_context
            .store
            .try_get_document_schema(&schema_uri)
            .await
        else {
            continue;
        };
        let Some(value_schema) = document_schema.value_schema.clone() else {
            continue;
        };
        return Some(CurrentSchema {
            value_schema,
            schema_uri: Cow::Owned(document_schema.schema_uri.clone()),
            definitions: Cow::Owned(document_schema.definitions.clone()),
        });
    }

    None
}
