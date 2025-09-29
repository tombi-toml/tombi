use std::{borrow::Cow, sync::Arc};

use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, DocumentSchema, OneOfSchema, PropertySchema,
    SchemaAccessor, ValueSchema,
};
use tombi_validator::Validate;

mod array;
mod array_of_table;
mod inline_table;
mod key_value;
mod root;
mod table;
mod value;

pub trait Edit {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>>;
}

fn edit_recursive<'a: 'b, 'b>(
    node: &'a tombi_document_tree::Value,
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
            return edit_recursive(
                node,
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
                        {
                            let current_schema = current_schema.into_owned();
                            if node
                                .validate(accessors.as_ref(), Some(&current_schema), schema_context)
                                .await
                                .is_ok()
                            {
                                return edit_recursive(
                                    node,
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

        let (accessors, value) = match (key_accessors.as_ref().first(), node) {
            (Some(Accessor::Key(key_str)), tombi_document_tree::Value::Table(table)) => {
                let mut accessors = accessors.as_ref().to_vec();
                accessors.push(Accessor::Key(key_str.to_owned()));
                let accessors: Arc<[Accessor]> = Arc::from(accessors.into_boxed_slice());

                let Some(value) = table.get(key_str) else {
                    return Vec::with_capacity(0);
                };

                (accessors, value)
            }
            (Some(Accessor::Index(index)), tombi_document_tree::Value::Array(array)) => {
                let mut accessors = accessors.as_ref().to_vec();
                accessors.push(Accessor::Index(*index));
                let accessors = Arc::from(accessors.into_boxed_slice());

                let Some(value) = array.get(*index) else {
                    return Vec::with_capacity(0);
                };

                (accessors, value)
            }
            (None, _) => return edit_fn(node, accessors, current_schema).await,
            _ => return Vec::with_capacity(0),
        };

        let key_accessors = &key_accessors[1..];

        if let Some(current_schema) = current_schema.as_ref() {
            match current_schema.value_schema.as_ref() {
                ValueSchema::Table(table_schema) => {
                    let Some(Accessor::Key(key_text)) = accessors.as_ref().last() else {
                        unreachable!("last accessor is not a key");
                    };
                    let key_schema_accessor = SchemaAccessor::Key(key_text.to_owned());

                    if let Some(PropertySchema {
                        property_schema, ..
                    }) = table_schema
                        .properties
                        .write()
                        .await
                        .get_mut(&key_schema_accessor)
                    {
                        if let Ok(Some(current_schema)) = property_schema
                            .resolve(
                                current_schema.schema_uri.clone(),
                                current_schema.definitions.clone(),
                                schema_context.store,
                            )
                            .await
                        {
                            return edit_recursive(
                                value,
                                edit_fn,
                                key_accessors,
                                accessors,
                                Some(current_schema.into_owned()),
                                schema_context,
                            )
                            .await;
                        }
                    }

                    if let Some(pattern_properties) = &table_schema.pattern_properties {
                        for (
                            property_key,
                            PropertySchema {
                                property_schema, ..
                            },
                        ) in pattern_properties.write().await.iter_mut()
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

                            if pattern.is_match(key_text) {
                                tracing::trace!("pattern_property_schema = {:?}", &property_schema);
                                if let Ok(Some(current_schema)) = property_schema
                                    .resolve(
                                        current_schema.schema_uri.clone(),
                                        current_schema.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                {
                                    return edit_recursive(
                                        value,
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

                    if let Some((_, referable_additional_property_schema)) =
                        &table_schema.additional_property_schema
                    {
                        tracing::trace!(
                            "additional_property_schema = {:?}",
                            referable_additional_property_schema
                        );

                        if let Ok(Some(current_schema)) = referable_additional_property_schema
                            .write()
                            .await
                            .resolve(
                                current_schema.schema_uri.clone(),
                                current_schema.definitions.clone(),
                                schema_context.store,
                            )
                            .await
                        {
                            return edit_recursive(
                                value,
                                edit_fn,
                                key_accessors,
                                accessors,
                                Some(current_schema.into_owned()),
                                schema_context,
                            )
                            .await;
                        };
                    }
                }
                ValueSchema::Array(array_schema) => {
                    if let Some(items) = &array_schema.items {
                        let mut item_schema = items.write().await;

                        if let Ok(Some(current_schema)) = item_schema
                            .resolve(
                                current_schema.schema_uri.clone(),
                                current_schema.definitions.clone(),
                                schema_context.store,
                            )
                            .await
                        {
                            return edit_recursive(
                                value,
                                edit_fn,
                                key_accessors,
                                accessors,
                                Some(current_schema.into_owned()),
                                schema_context,
                            )
                            .await;
                        };
                    }
                }
                _ => {}
            }
        }

        edit_recursive(
            value,
            edit_fn,
            key_accessors,
            accessors,
            None,
            schema_context,
        )
        .await
    }
    .boxed()
}
