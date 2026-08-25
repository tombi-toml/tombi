use itertools::Itertools;

use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaAccessor, SchemaView, TableSchema};

use crate::{
    comment_directive::get_table_comment_directive_content_with_schema_uri,
    goto_type_definition::{
        GetTypeDefinition, TypeDefinition, all_of::get_all_of_type_definition,
        any_of::get_any_of_type_definition,
        comment::get_tombi_value_comment_directive_type_definition,
        one_of::get_one_of_type_definition,
    },
    schema_resolver::resolve_table_unevaluated_property_schema,
};

impl GetTypeDefinition for tombi_document_tree_syntax::Table {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        log::trace!("self = {:?}", self);
        log::trace!("keys = {:?}", keys);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("current_schema = {:?}", current_schema);

        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_table_comment_directive_content_with_schema_uri(self, position, accessors)
                && let hover_content = get_tombi_value_comment_directive_type_definition(
                    comment_directive_context,
                    schema_uri,
                )
                .await
                && !hover_content.is_empty()
            {
                return hover_content;
            }

            if let Some(Ok(current_schema)) = schema_context
                .get_subschema(accessors, current_schema)
                .await
            {
                return self
                    .get_type_definition(
                        position,
                        keys,
                        accessors,
                        Some(&current_schema),
                        schema_context,
                    )
                    .await;
            }

            if let Some(current_schema) = current_schema {
                match current_schema.schema_view.as_ref() {
                    SchemaView::Table(table_schema) => {
                        if let Some(key) = keys.first() {
                            if let Some(value) = self.get(key) {
                                let accessor = Accessor::Key(key.value().to_owned());
                                let schema_accessor = SchemaAccessor::from(&accessor);
                                let accessors = accessors
                                    .iter()
                                    .cloned()
                                    .chain(std::iter::once(accessor))
                                    .collect_vec();

                                let key_range = {
                                    let properties = table_schema.properties.read().await;
                                    properties
                                        .get(&schema_accessor)
                                        .map(|property_schema| property_schema.key_range)
                                };
                                if let Some(key_range) = key_range {
                                    if let Ok(Some(current_schema)) = table_schema
                                        .resolve_property_schema(
                                            &schema_accessor,
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            current_schema.strict,
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        if tombi_document_tree_syntax::ValueImpl::range(key)
                                            .contains(position)
                                        {
                                            return current_schema
                                                .schema_view
                                                .get_type_definition(
                                                    position,
                                                    &keys[1..],
                                                    &accessors,
                                                    Some(&current_schema),
                                                    schema_context,
                                                )
                                                .await
                                                .into_iter()
                                                .map(|type_definition| {
                                                    type_definition
                                                        .update_range(&accessors, &key_range)
                                                })
                                                .collect();
                                        }
                                        return value
                                            .get_type_definition(
                                                position,
                                                &keys[1..],
                                                &accessors,
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await
                                            .into_iter()
                                            .map(|type_definition| {
                                                type_definition.update_range(&accessors, &key_range)
                                            })
                                            .collect();
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
                                    let pattern_properties = pattern_properties
                                        .read()
                                        .await
                                        .iter()
                                        .map(|(key, property_schema)| {
                                            (key.to_string(), property_schema.key_range)
                                        })
                                        .collect_vec();
                                    for (property_key, key_range) in pattern_properties {
                                        if let Ok(pattern) = tombi_regex::Regex::new(&property_key)
                                        {
                                            if pattern.is_match(key.value()) {
                                                if let Ok(Some(current_schema)) = table_schema
                                                    .resolve_pattern_property_schema(
                                                        &property_key,
                                                        current_schema.schema_uri.clone(),
                                                        current_schema.definitions.clone(),
                                                        current_schema.strict,
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
                                                        .into_iter()
                                                        .map(|type_definition| {
                                                            type_definition.update_range(
                                                                &accessors, &key_range,
                                                            )
                                                        })
                                                        .collect();
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
                                            log::warn!(
                                                "invalid regex pattern property: {}",
                                                property_key
                                            );
                                        };
                                    }
                                }

                                if let Some((
                                    schema_key_range,
                                    referable_additional_property_schema,
                                )) = &table_schema.additional_property_schema
                                    && let Ok(Some(current_schema)) =
                                        tombi_schema_store::resolve_schema_item(
                                            referable_additional_property_schema,
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            current_schema.strict,
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
                                        .into_iter()
                                        .map(|type_definition| {
                                            type_definition
                                                .update_range(&accessors, schema_key_range)
                                        })
                                        .collect();
                                }

                                if let Some(one_of_schema) = table_schema.one_of.as_deref()
                                    && let type_definitions = get_one_of_type_definition(
                                        self,
                                        position,
                                        keys,
                                        &accessors,
                                        one_of_schema,
                                        &current_schema.schema_uri,
                                        &current_schema.definitions,
                                        current_schema.strict,
                                        schema_context,
                                    )
                                    .await
                                    && !type_definitions.is_empty()
                                {
                                    return type_definitions;
                                }
                                if let Some(any_of_schema) = table_schema.any_of.as_deref()
                                    && let type_definitions = get_any_of_type_definition(
                                        self,
                                        position,
                                        keys,
                                        &accessors,
                                        any_of_schema,
                                        &current_schema.schema_uri,
                                        &current_schema.definitions,
                                        current_schema.strict,
                                        schema_context,
                                    )
                                    .await
                                    && !type_definitions.is_empty()
                                {
                                    return type_definitions;
                                }
                                if let Some(all_of_schema) = table_schema.all_of.as_deref()
                                    && let type_definitions = get_all_of_type_definition(
                                        self,
                                        position,
                                        keys,
                                        &accessors,
                                        all_of_schema,
                                        &current_schema.schema_uri,
                                        &current_schema.definitions,
                                        current_schema.strict,
                                        schema_context,
                                    )
                                    .await
                                    && !type_definitions.is_empty()
                                {
                                    return type_definitions;
                                }

                                if let Some(current_schema) =
                                    resolve_table_unevaluated_property_schema(
                                        table_schema,
                                        current_schema,
                                        schema_context,
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
                                        .await;
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

                                vec![TypeDefinition {
                                    schema_uri,
                                    schema_accessors: accessors
                                        .iter()
                                        .map(Into::into)
                                        .collect_vec(),
                                    range: tombi_text::Range::default(),
                                }]
                            }
                        } else {
                            let type_definition = table_schema
                                .get_type_definition(
                                    position,
                                    keys,
                                    accessors,
                                    Some(current_schema),
                                    schema_context,
                                )
                                .await;

                            if !type_definition.is_empty() {
                                return type_definition;
                            }

                            if let Some(one_of_schema) = table_schema.one_of.as_deref()
                                && let type_definitions = get_one_of_type_definition(
                                    self,
                                    position,
                                    keys,
                                    accessors,
                                    one_of_schema,
                                    &current_schema.schema_uri,
                                    &current_schema.definitions,
                                    current_schema.strict,
                                    schema_context,
                                )
                                .await
                                && !type_definitions.is_empty()
                            {
                                return type_definitions;
                            }
                            if let Some(any_of_schema) = table_schema.any_of.as_deref()
                                && let type_definitions = get_any_of_type_definition(
                                    self,
                                    position,
                                    keys,
                                    accessors,
                                    any_of_schema,
                                    &current_schema.schema_uri,
                                    &current_schema.definitions,
                                    current_schema.strict,
                                    schema_context,
                                )
                                .await
                                && !type_definitions.is_empty()
                            {
                                return type_definitions;
                            }
                            if let Some(all_of_schema) = table_schema.all_of.as_deref()
                                && let type_definitions = get_all_of_type_definition(
                                    self,
                                    position,
                                    keys,
                                    accessors,
                                    all_of_schema,
                                    &current_schema.schema_uri,
                                    &current_schema.definitions,
                                    current_schema.strict,
                                    schema_context,
                                )
                                .await
                                && !type_definitions.is_empty()
                            {
                                return type_definitions;
                            }

                            Vec::new()
                        }
                    }
                    SchemaView::OneOf(one_of_schema) => {
                        get_one_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            current_schema.strict,
                            schema_context,
                        )
                        .await
                    }
                    SchemaView::AnyOf(any_of_schema) => {
                        get_any_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            current_schema.strict,
                            schema_context,
                        )
                        .await
                    }
                    SchemaView::AllOf(all_of_schema) => {
                        get_all_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            current_schema.strict,
                            schema_context,
                        )
                        .await
                    }
                    _ => vec![TypeDefinition {
                        schema_uri: current_schema.schema_uri.as_ref().clone(),
                        schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                        range: tombi_text::Range::default(),
                    }],
                }
            } else {
                if let Some(key) = keys.first()
                    && let Some(value) = self.get(key)
                {
                    let accessor = Accessor::Key(key.value().to_owned());

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
                Vec::new()
            }
        }
        .boxed()
    }
}

impl GetTypeDefinition for TableSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        async move {
            current_schema.map_or_else(Vec::new, |schema| {
                let mut schema_uri = schema.schema_uri.as_ref().clone();
                schema_uri.set_fragment(Some(&format!("L{}", self.range.start.line + 1)));

                vec![TypeDefinition {
                    schema_uri,
                    schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                    range: schema.schema_view.range(),
                }]
            })
        }
        .boxed()
    }
}
