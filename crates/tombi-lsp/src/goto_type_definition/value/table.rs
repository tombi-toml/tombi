use std::borrow::Cow;

use itertools::Itertools;

use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, CurrentSchema, DocumentSchema, PropertySchema, SchemaAccessor, TableSchema,
    ValueSchema,
};

use crate::{
    comment_directive::get_table_comment_directive_content_with_schema_uri,
    goto_type_definition::{
        all_of::get_all_of_type_definition, any_of::get_any_of_type_definition,
        comment::get_tombi_value_comment_directive_type_definition,
        one_of::get_one_of_type_definition, GetTypeDefinition, TypeDefinition,
    },
};

impl GetTypeDefinition for tombi_document_tree::Table {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<TypeDefinition>> {
        tracing::trace!("self = {:?}", self);
        tracing::trace!("keys = {:?}", keys);
        tracing::trace!("accessors = {:?}", accessors);
        tracing::trace!("current_schema = {:?}", current_schema);

        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_table_comment_directive_content_with_schema_uri(self, position, accessors)
            {
                if let Some(hover_content) = get_tombi_value_comment_directive_type_definition(
                    comment_directive_context,
                    schema_uri,
                )
                .await
                {
                    return Some(hover_content);
                }
            }

            if let Some(Ok(DocumentSchema {
                value_schema,
                schema_uri,
                definitions,
                ..
            })) = schema_context
                .get_subschema(accessors, current_schema)
                .await
            {
                let current_schema = value_schema.map(|value_schema| CurrentSchema {
                    value_schema: Cow::Owned(value_schema),
                    schema_uri: Cow::Owned(schema_uri),
                    definitions: Cow::Owned(definitions),
                });

                return self
                    .get_type_definition(
                        position,
                        keys,
                        accessors,
                        current_schema.as_ref(),
                        schema_context,
                    )
                    .await;
            }

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Table(table_schema) => {
                        if let Some(key) = keys.first() {
                            if let Some(value) = self.get(key) {
                                let accessor = Accessor::Key(key.value.clone());
                                let schema_accessor = SchemaAccessor::from(&accessor);
                                let accessors = accessors
                                    .iter()
                                    .cloned()
                                    .chain(std::iter::once(accessor))
                                    .collect_vec();

                                if let Some(PropertySchema {
                                    key_range,
                                    property_schema,
                                    ..
                                }) = table_schema
                                    .properties
                                    .write()
                                    .await
                                    .get_mut(&schema_accessor)
                                {
                                    tracing::trace!("property_schema = {:?}", property_schema);

                                    if let Ok(Some(current_schema)) = property_schema
                                        .resolve(
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        return value
                                            .get_type_definition(
                                                position,
                                                &keys[1..],
                                                &accessors,
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await
                                            .map(|type_definition| {
                                                type_definition.update_range(&accessors, key_range)
                                            });
                                    }

                                    return value
                                        .get_type_definition(
                                            position,
                                            &keys[1..],
                                            &accessors,
                                            None,
                                            schema_context,
                                        )
                                        .await;
                                }
                                if let Some(pattern_properties) = &table_schema.pattern_properties {
                                    for (
                                        property_key,
                                        PropertySchema {
                                            property_schema,
                                            key_range,
                                            ..
                                        },
                                    ) in pattern_properties.write().await.iter_mut()
                                    {
                                        if let Ok(pattern) = regex::Regex::new(property_key) {
                                            if pattern.is_match(&key.value) {
                                                if let Ok(Some(current_schema)) = property_schema
                                                    .resolve(
                                                        current_schema.schema_uri.clone(),
                                                        current_schema.definitions.clone(),
                                                        schema_context.store,
                                                    )
                                                    .await
                                                {
                                                    return value
                                                        .get_type_definition(
                                                            position,
                                                            &keys[1..],
                                                            &accessors,
                                                            Some(&current_schema),
                                                            schema_context,
                                                        )
                                                        .await
                                                        .map(|type_definition| {
                                                            type_definition
                                                                .update_range(&accessors, key_range)
                                                        });
                                                }

                                                return value
                                                    .get_type_definition(
                                                        position,
                                                        &keys[1..],
                                                        &accessors,
                                                        None,
                                                        schema_context,
                                                    )
                                                    .await;
                                            }
                                        } else {
                                            tracing::warn!(
                                                "Invalid regex pattern property: {}",
                                                property_key
                                            );
                                        };
                                    }
                                }

                                if let Some((
                                    schema_key_range,
                                    referable_additional_property_schema,
                                )) = &table_schema.additional_property_schema
                                {
                                    let mut referable_schema =
                                        referable_additional_property_schema.write().await;
                                    if let Ok(Some(current_schema)) = referable_schema
                                        .resolve(
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        return value
                                            .get_type_definition(
                                                position,
                                                &keys[1..],
                                                &accessors,
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await
                                            .map(|type_definition| {
                                                type_definition
                                                    .update_range(&accessors, schema_key_range)
                                            });
                                    }
                                }

                                value
                                    .get_type_definition(
                                        position,
                                        &keys[1..],
                                        &accessors,
                                        None,
                                        schema_context,
                                    )
                                    .await
                            } else {
                                let mut schema_uri = current_schema.schema_uri.as_ref().clone();
                                schema_uri.set_fragment(Some(&format!(
                                    "L{}",
                                    key.range().start.line + 1
                                )));

                                Some(TypeDefinition {
                                    schema_uri,
                                    schema_accessors: accessors
                                        .iter()
                                        .map(Into::into)
                                        .collect_vec(),
                                    range: tombi_text::Range::default(),
                                })
                            }
                        } else {
                            table_schema
                                .get_type_definition(
                                    position,
                                    keys,
                                    accessors,
                                    Some(current_schema),
                                    schema_context,
                                )
                                .await
                        }
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        get_one_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            schema_context,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        get_any_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            schema_context,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        get_all_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            schema_context,
                        )
                        .await
                    }
                    _ => Some(TypeDefinition {
                        schema_uri: current_schema.schema_uri.as_ref().clone(),
                        schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                        range: tombi_text::Range::default(),
                    }),
                }
            } else {
                if let Some(key) = keys.first() {
                    if let Some(value) = self.get(key) {
                        let accessor = Accessor::Key(key.value.clone());

                        return value
                            .get_type_definition(
                                position,
                                &keys[1..],
                                &accessors
                                    .iter()
                                    .cloned()
                                    .chain(std::iter::once(accessor))
                                    .collect_vec(),
                                None,
                                schema_context,
                            )
                            .await;
                    }
                }
                None
            }
        }
        .boxed()
    }
}

impl GetTypeDefinition for TableSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<TypeDefinition>> {
        async move {
            current_schema.map(|schema| {
                let mut schema_uri = schema.schema_uri.as_ref().clone();
                schema_uri.set_fragment(Some(&format!("L{}", self.range.start.line + 1)));

                TypeDefinition {
                    schema_uri,
                    schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                    range: schema.value_schema.range(),
                }
            })
        }
        .boxed()
    }
}
