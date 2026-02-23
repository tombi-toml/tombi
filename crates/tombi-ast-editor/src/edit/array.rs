use std::borrow::Cow;

use itertools::Itertools;
use tombi_ast::{AstNode, DanglingCommentGroupOr};
use tombi_comment_directive::value::{ArrayCommonFormatRules, ArrayCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, OneOfSchema, ValueSchema,
};
use tombi_validator::Validate;

use crate::rule::{array_comma_trailing_comment, array_values_order};

impl crate::Edit for tombi_ast::Array {
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
            let tombi_document_tree::Value::Array(array_node) = node else {
                return Vec::with_capacity(0);
            };
            let current_item_schema = resolve_array_item_edit_context(
                array_node,
                accessors,
                current_schema.cloned(),
                schema_context,
            )
            .await;

            let mut changes = vec![];
            let mut value_nodes_iter = array_node.values().iter().enumerate();

            for group in self.value_with_comma_groups() {
                let DanglingCommentGroupOr::ItemGroup(value_group) = group else {
                    continue;
                };

                for ((value, comma), (index, value_node)) in value_group
                    .values_with_comma()
                    .zip(value_nodes_iter.by_ref())
                {
                    changes.extend(array_comma_trailing_comment(&value, comma.as_ref()));
                    changes.extend(
                        value
                            .edit(
                                value_node,
                                &accessors
                                    .iter()
                                    .cloned()
                                    .chain(std::iter::once(Accessor::Index(index)))
                                    .collect_vec(),
                                source_path,
                                current_item_schema.as_ref(),
                                schema_context,
                            )
                            .await,
                    );
                }
            }

            let comment_directive =
                get_comment_directive_content::<ArrayCommonFormatRules, ArrayCommonLintRules>(
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

            let array_schema_values_order = current_schema.and_then(|current_schema| {
                if let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref() {
                    array_schema.values_order.clone()
                } else {
                    None
                }
            });
            let mut nodes_iter = array_node.values().iter().enumerate();
            for group in self.value_with_comma_groups() {
                let DanglingCommentGroupOr::ItemGroup(value_group) = group else {
                    continue;
                };

                let values_with_comma = value_group.values_with_comma().collect_vec();
                let nodes = nodes_iter
                    .by_ref()
                    .take(values_with_comma.len())
                    .collect_vec();

                changes.extend(
                    array_values_order(
                        nodes,
                        values_with_comma,
                        accessors,
                        current_item_schema.as_ref(),
                        schema_context,
                        array_schema_values_order.clone(),
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

fn resolve_array_item_edit_context<'a: 'b, 'b>(
    node: &'a tombi_document_tree::Array,
    accessors: &'a [Accessor],
    current_schema: Option<tombi_schema_store::CurrentSchema<'a>>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> BoxFuture<'b, Option<tombi_schema_store::CurrentSchema<'a>>> {
    async move {
        if let Some(Ok(document_schema)) = schema_context
            .get_subschema(accessors, current_schema.as_ref())
            .await
            && let Some(value_schema) = &document_schema.value_schema
        {
            return resolve_array_item_edit_context(
                node,
                accessors,
                Some(CurrentSchema {
                    value_schema: value_schema.clone(),
                    schema_uri: Cow::Owned(document_schema.schema_uri.clone()),
                    definitions: Cow::Owned(document_schema.definitions.clone()),
                }),
                schema_context,
            )
            .await;
        }

        if let Some(current_schema) = current_schema.as_ref() {
            match current_schema.value_schema.as_ref() {
                ValueSchema::AllOf(AllOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                | ValueSchema::OneOf(OneOfSchema { schemas, .. }) => {
                    if let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
                        schemas,
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                        &schema_context.schema_visits,
                        accessors,
                    )
                    .await
                    {
                        for current_schema in resolved_schemas {
                            if node
                                .validate(accessors, Some(&current_schema), schema_context)
                                .await
                                .is_ok()
                            {
                                return resolve_array_item_edit_context(
                                    node,
                                    accessors,
                                    Some(current_schema),
                                    schema_context,
                                )
                                .await;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(current_schema) = current_schema.as_ref()
            && let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref()
            && let Some(item_schema) = &array_schema.items
            && let Ok(Some(current_schema)) = tombi_schema_store::resolve_schema_item(
                item_schema,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
            .inspect_err(|err| log::warn!("{err}"))
        {
            return Some(current_schema);
        }

        None
    }
    .boxed()
}
