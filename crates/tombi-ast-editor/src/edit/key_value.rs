use std::{borrow::Cow, sync::Arc};

use itertools::Itertools;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, DocumentSchema, OneOfSchema, PropertySchema,
    SchemaAccessor, ValueSchema,
};
use tombi_validator::Validate;

use crate::edit::EditRecursive;

impl crate::Edit for tombi_ast::KeyValue {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            tracing::trace!("node = {:?}", node);
            tracing::trace!("accessors = {:?}", accessors);
            tracing::trace!("current_schema = {:?}", current_schema);

            let Some(keys_accessors) = self.get_accessors(schema_context.toml_version) else {
                return Vec::with_capacity(0);
            };

            let key_accessors = accessors
                .iter()
                .cloned()
                .chain(keys_accessors)
                .collect_vec();

            node.edit_recursive(
                |node, accessors, current_schema| {
                    async move {
                        tracing::trace!("node = {:?}", node);
                        tracing::trace!("accessors = {:?}", accessors);
                        tracing::trace!("current_schema = {:?}", current_schema);

                        if let Some(value) = self.value() {
                            value
                                .edit(
                                    node,
                                    &accessors,
                                    source_path,
                                    current_schema.as_ref(),
                                    schema_context,
                                )
                                .await
                        } else {
                            Vec::with_capacity(0)
                        }
                    }
                    .boxed()
                },
                &key_accessors,
                Arc::from(accessors.to_vec()),
                current_schema.map(|current_schema| current_schema.clone()),
                schema_context,
            )
            .await
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Value {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        edit_fn: impl FnOnce(
                &'a tombi_document_tree::Value,
                Arc<[Accessor]>,
                Option<tombi_schema_store::CurrentSchema<'a>>,
            ) -> BoxFuture<'b, Vec<crate::Change>>
            + std::marker::Send
            + 'b,
        key_accessors: &'a [Accessor],
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
                return self
                    .edit_recursive(
                        edit_fn,
                        key_accessors,
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

            match (key_accessors.as_ref().first(), self) {
                (Some(Accessor::Key(key_str)), tombi_document_tree::Value::Table(table)) => {
                    let mut next_accessors = accessors.as_ref().to_vec();
                    next_accessors.push(Accessor::Key(key_str.to_owned()));
                    let next_accessors = Arc::from(next_accessors.into_boxed_slice());

                    if key_accessors.len() == 1 {
                        edit_fn(self, accessors, current_schema).await
                    } else {
                        table
                            .edit_recursive(
                                edit_fn,
                                &key_accessors[1..],
                                next_accessors,
                                current_schema,
                                schema_context,
                            )
                            .await
                    }
                }
                (Some(Accessor::Index(index)), tombi_document_tree::Value::Array(array)) => {
                    let mut next_accessors = accessors.as_ref().to_vec();
                    next_accessors.push(Accessor::Index(*index));
                    let next_accessors = Arc::from(next_accessors.into_boxed_slice());

                    if key_accessors.len() == 1 {
                        edit_fn(self, accessors, current_schema).await
                    } else {
                        array
                            .edit_recursive(
                                edit_fn,
                                &key_accessors[1..],
                                next_accessors,
                                current_schema,
                                schema_context,
                            )
                            .await
                    }
                }
                (None, _) => edit_fn(self, accessors, current_schema).await,
                _ => Vec::with_capacity(0),
            }
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Table {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        edit_fn: impl FnOnce(
                &'a tombi_document_tree::Value,
                Arc<[Accessor]>,
                Option<tombi_schema_store::CurrentSchema<'a>>,
            ) -> BoxFuture<'b, Vec<crate::Change>>
            + std::marker::Send
            + 'b,
        key_accessors: &'a [Accessor],
        accessors: Arc<[Accessor]>,
        current_schema: Option<tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let Some(Accessor::Key(accessor_str)) = key_accessors.as_ref().first() else {
                unreachable!("key_accessors is empty");
            };
            let Some(value) = self.get(accessor_str) else {
                return Vec::with_capacity(0);
            };

            let current_schema = current_schema.map(|current_schema| CurrentSchema {
                value_schema: current_schema.value_schema.to_owned(),
                schema_uri: current_schema.schema_uri.to_owned(),
                definitions: current_schema.definitions.to_owned(),
            });

            if let Some(current_schema_ref) = current_schema.as_ref() {
                match current_schema_ref.value_schema.as_ref() {
                    ValueSchema::Table(table_schema) => {
                        let key_accessor = SchemaAccessor::Key(accessor_str.to_owned());

                        let resolved_schema = {
                            let mut properties = table_schema.properties.write().await;

                            if let Some(PropertySchema {
                                property_schema, ..
                            }) = properties.get_mut(&key_accessor)
                            {
                                if let Ok(Some(current_schema)) = property_schema
                                    .resolve(
                                        current_schema_ref.schema_uri.clone(),
                                        current_schema_ref.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                {
                                    Some(current_schema.into_owned())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };

                        if let Some(resolved_schema) = resolved_schema {
                            return value
                                .edit_recursive(
                                    edit_fn,
                                    key_accessors,
                                    accessors,
                                    Some(resolved_schema),
                                    schema_context,
                                )
                                .await;
                        }

                        if let Some(pattern_properties) = &table_schema.pattern_properties {
                            let resolved_schema = {
                                let mut pattern_properties = pattern_properties.write().await;
                                let mut resolved = None;

                                for (
                                    property_key,
                                    PropertySchema {
                                        property_schema, ..
                                    },
                                ) in pattern_properties.iter_mut()
                                {
                                    let pattern = match regex::Regex::new(property_key) {
                                        Ok(pattern) => pattern,
                                        Err(_) => {
                                            tracing::warn!(
                                                "Invalid regex pattern property: {}",
                                                property_key
                                            );
                                            continue;
                                        }
                                    };

                                    if pattern.is_match(accessor_str) {
                                        tracing::trace!(
                                            "pattern_property_schema = {:?}",
                                            &property_schema
                                        );
                                        if let Ok(Some(current_schema)) = property_schema
                                            .resolve(
                                                current_schema_ref.schema_uri.clone(),
                                                current_schema_ref.definitions.clone(),
                                                schema_context.store,
                                            )
                                            .await
                                        {
                                            resolved = Some(current_schema.into_owned());
                                            break;
                                        }
                                    }
                                }

                                resolved
                            };

                            if let Some(resolved_schema) = resolved_schema {
                                return value
                                    .edit_recursive(
                                        edit_fn,
                                        key_accessors,
                                        accessors,
                                        Some(resolved_schema),
                                        schema_context,
                                    )
                                    .await;
                            }
                        }

                        if let Some((_, referable_additional_property_schema)) =
                            &table_schema.additional_property_schema
                        {
                            tracing::trace!(
                                "additional_property_schema = {:?}",
                                referable_additional_property_schema
                            );

                            let resolved_schema = {
                                let mut schema = referable_additional_property_schema.write().await;
                                if let Ok(Some(current_schema)) = schema
                                    .resolve(
                                        current_schema_ref.schema_uri.clone(),
                                        current_schema_ref.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                {
                                    Some(current_schema.into_owned())
                                } else {
                                    None
                                }
                            };

                            if let Some(resolved_schema) = resolved_schema {
                                return value
                                    .edit_recursive(
                                        edit_fn,
                                        key_accessors,
                                        accessors,
                                        Some(resolved_schema),
                                        schema_context,
                                    )
                                    .await;
                            }
                        }
                    }
                    ValueSchema::AllOf(AllOfSchema { schemas, .. })
                    | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                    | ValueSchema::OneOf(OneOfSchema { schemas, .. }) => {
                        for referable_schema in schemas.write().await.iter_mut() {
                            if let Ok(Some(current_schema)) = referable_schema
                                .resolve(
                                    current_schema_ref.schema_uri.clone(),
                                    current_schema_ref.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                let current_schema = current_schema.into_owned();
                                if self
                                    .validate(
                                        accessors.as_ref(),
                                        Some(&current_schema),
                                        schema_context,
                                    )
                                    .await
                                    .is_ok()
                                {
                                    return value
                                        .edit_recursive(
                                            edit_fn,
                                            key_accessors,
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

            value
                .edit_recursive(edit_fn, key_accessors, accessors, None, schema_context)
                .await
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Array {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        edit_fn: impl FnOnce(
                &'a tombi_document_tree::Value,
                Arc<[Accessor]>,
                Option<tombi_schema_store::CurrentSchema<'a>>,
            ) -> BoxFuture<'b, Vec<crate::Change>>
            + std::marker::Send
            + 'b,
        key_accessors: &'a [Accessor],
        accessors: Arc<[Accessor]>,
        current_schema: Option<tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let Some(Accessor::Index(index)) = key_accessors.as_ref().first() else {
                return Vec::with_capacity(0);
            };
            let Some(value) = self.get(*index) else {
                return Vec::with_capacity(0);
            };

            let current_schema = current_schema.map(|current_schema| CurrentSchema {
                value_schema: current_schema.value_schema.to_owned(),
                schema_uri: current_schema.schema_uri.to_owned(),
                definitions: current_schema.definitions.to_owned(),
            });

            if let Some(current_schema_ref) = current_schema.as_ref() {
                match current_schema_ref.value_schema.as_ref() {
                    ValueSchema::Array(array_schema) => {
                        if let Some(items) = &array_schema.items {
                            let resolved_schema = {
                                let mut item_schema = items.write().await;

                                if let Ok(Some(current_schema)) = item_schema
                                    .resolve(
                                        current_schema_ref.schema_uri.clone(),
                                        current_schema_ref.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                {
                                    Some(current_schema.into_owned())
                                } else {
                                    None
                                }
                            };

                            if let Some(resolved_schema) = resolved_schema {
                                return value
                                    .edit_recursive(
                                        edit_fn,
                                        key_accessors,
                                        accessors,
                                        Some(resolved_schema),
                                        schema_context,
                                    )
                                    .await;
                            }
                        }
                    }
                    ValueSchema::AllOf(AllOfSchema { schemas, .. })
                    | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                    | ValueSchema::OneOf(OneOfSchema { schemas, .. }) => {
                        for referable_schema in schemas.write().await.iter_mut() {
                            if let Ok(Some(current_schema)) = referable_schema
                                .resolve(
                                    current_schema_ref.schema_uri.clone(),
                                    current_schema_ref.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                let current_schema = current_schema.into_owned();

                                if self
                                    .validate(
                                        accessors.as_ref(),
                                        Some(&current_schema),
                                        schema_context,
                                    )
                                    .await
                                    .is_ok()
                                {
                                    return value
                                        .edit_recursive(
                                            edit_fn,
                                            key_accessors,
                                            accessors,
                                            Some(current_schema.into_owned()),
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

            value
                .edit_recursive(edit_fn, key_accessors, accessors, None, schema_context)
                .await
        }
        .boxed()
    }
}
