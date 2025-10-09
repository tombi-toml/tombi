use std::{borrow::Cow, sync::Arc};

use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_comment_directive::value::{ArrayCommonFormatRules, ArrayCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, DocumentSchema, OneOfSchema, ValueSchema,
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
        tracing::trace!("node = {:?}", node);
        tracing::trace!("accessors = {:?}", accessors);
        tracing::trace!("current_schema = {:?}", current_schema);

        async move {
            let tombi_document_tree::Value::Array(array_node) = node else {
                return Vec::with_capacity(0);
            };

            let array_schema_values_order = if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::AllOf(AllOfSchema { schemas, .. })
                    | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                    | ValueSchema::OneOf(OneOfSchema { schemas, .. }) => {
                        for referable_schema in schemas.write().await.iter_mut() {
                            if let Ok(Some(current_schema)) = referable_schema
                                .resolve(
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                                .inspect_err(|err| tracing::warn!("{err}"))
                            {
                                if array_node
                                    .validate(
                                        accessors.as_ref(),
                                        Some(&current_schema),
                                        schema_context,
                                    )
                                    .await
                                    .is_ok()
                                {
                                    return self
                                        .edit(
                                            node,
                                            accessors,
                                            source_path,
                                            Some(&current_schema),
                                            schema_context,
                                        )
                                        .await;
                                }
                            }
                        }
                        None
                    }
                    ValueSchema::Array(array_schema) => array_schema.values_order.clone(),
                    _ => None,
                }
            } else {
                None
            };

            edit_item(
                array_node,
                |node, accessors, current_schema| {
                    async move {
                        tracing::trace!("node = {:?}", node);
                        tracing::trace!("accessors = {:?}", accessors);
                        tracing::trace!("current_schema = {:?}", current_schema);

                        let mut changes = vec![];
                        for (index, ((value, comma), value_node)) in
                            self.values_with_comma().zip(node.values()).enumerate()
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
                                        current_schema.as_ref(),
                                        schema_context,
                                    )
                                    .await,
                            );
                        }

                        let comment_directive = get_comment_directive_content::<
                            ArrayCommonFormatRules,
                            ArrayCommonLintRules,
                        >(
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

                        changes.extend(
                            array_values_order(
                                self.values_with_comma().collect_vec(),
                                array_node,
                                &accessors,
                                current_schema.as_ref(),
                                schema_context,
                                array_schema_values_order,
                                comment_directive,
                            )
                            .await,
                        );

                        changes
                    }
                    .boxed()
                },
                Arc::from(accessors.to_vec()),
                current_schema.cloned(),
                schema_context,
            )
            .await
        }
        .boxed()
    }
}

fn edit_item<'a: 'b, 'b>(
    node: &'a tombi_document_tree::Array,
    edit_fn: impl FnOnce(
            &'a tombi_document_tree::Array,
            Arc<[Accessor]>,
            Option<tombi_schema_store::CurrentSchema<'a>>,
        ) -> BoxFuture<'b, Vec<crate::Change>>
        + std::marker::Send
        + 'b,
    accessors: Arc<[Accessor]>,
    current_schema: Option<tombi_schema_store::CurrentSchema<'a>>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> BoxFuture<'b, Vec<crate::Change>> {
    async move {
        if let Some(Ok(DocumentSchema {
            value_schema: Some(value_schema),
            schema_uri,
            definitions,
            ..
        })) = schema_context
            .get_subschema(accessors.as_ref(), current_schema.as_ref())
            .await
        {
            return edit_item(
                node,
                edit_fn,
                accessors,
                Some(CurrentSchema {
                    value_schema: Cow::Owned(value_schema),
                    schema_uri: Cow::Owned(schema_uri),
                    definitions: Cow::Owned(definitions),
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
                    for referable_schema in schemas.write().await.iter_mut() {
                        if let Ok(Some(current_schema)) = referable_schema
                            .resolve(
                                current_schema.schema_uri.clone(),
                                current_schema.definitions.clone(),
                                schema_context.store,
                            )
                            .await
                            .inspect_err(|err| tracing::warn!("{err}"))
                        {
                            let current_schema = current_schema.into_owned();
                            if node
                                .validate(accessors.as_ref(), Some(&current_schema), schema_context)
                                .await
                                .is_ok()
                            {
                                return edit_item(
                                    node,
                                    edit_fn,
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

        if let Some(current_schema) = current_schema.as_ref() {
            if let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref() {
                if let Some(item_schema) = &array_schema.items {
                    if let Ok(Some(current_schema)) = item_schema
                        .write()
                        .await
                        .resolve(
                            current_schema.schema_uri.clone(),
                            current_schema.definitions.clone(),
                            schema_context.store,
                        )
                        .await
                        .inspect_err(|err| tracing::warn!("{err}"))
                    {
                        return edit_fn(node, accessors, Some(current_schema.into_owned())).await;
                    }
                }
            }
        }

        edit_fn(node, accessors, None).await
    }
    .boxed()
}
