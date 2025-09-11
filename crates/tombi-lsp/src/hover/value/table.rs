use std::borrow::Cow;

use itertools::Itertools;

use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, Accessors, CurrentSchema, DocumentSchema, PropertySchema, SchemaAccessor,
    TableSchema, ValueSchema, ValueType,
};

use crate::{
    comment_directive::get_table_comment_directive_content_with_schema_uri,
    hover::{
        all_of::get_all_of_hover_content,
        any_of::get_any_of_hover_content,
        comment::get_value_comment_directive_hover_content,
        constraints::{build_enumerate_values, ValueConstraints},
        one_of::get_one_of_hover_content,
        GetHoverContent, HoverValueContent,
    },
    HoverContent,
};

impl GetHoverContent for tombi_document_tree::Table {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        tracing::trace!("self = {:?}", self);
        tracing::trace!("keys = {:?}", keys);
        tracing::trace!("accessors = {:?}", accessors);
        tracing::trace!("current_schema = {:?}", current_schema);

        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_table_comment_directive_content_with_schema_uri(self, position, accessors)
            {
                if let Some(hover_content) =
                    get_value_comment_directive_hover_content(comment_directive_context, schema_uri)
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
                    .get_hover_content(
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
                                let key_patterns = match table_schema.pattern_properties.as_ref() {
                                    Some(pattern_properties) => Some(
                                        pattern_properties
                                            .read()
                                            .await
                                            .keys()
                                            .map(ToString::to_string)
                                            .collect_vec(),
                                    ),
                                    None => None,
                                };

                                if let Some(PropertySchema {
                                    property_schema, ..
                                }) = table_schema
                                    .properties
                                    .write()
                                    .await
                                    .get_mut(&SchemaAccessor::from(&accessor))
                                {
                                    tracing::trace!("property_schema = {:?}", property_schema);
                                    let required = table_schema
                                        .required
                                        .as_ref()
                                        .map(|r| r.contains(&key.value))
                                        .unwrap_or(false);

                                    if let Ok(Some(current_schema)) = property_schema
                                        .resolve(
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        let mut hover_content = value
                                            .get_hover_content(
                                                position,
                                                &keys[1..],
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor))
                                                    .collect_vec(),
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await;
                                        if let Some(HoverContent::Value(hover_value_content)) =
                                            hover_content.as_mut()
                                        {
                                            if keys.len() == 1 {
                                                // Check if cursor is not on the value
                                                if !value.contains(position) {
                                                    // When cursor is on key or equals sign,
                                                    // use the property's title and description
                                                    if let Some(title) =
                                                        current_schema.value_schema.title()
                                                    {
                                                        hover_value_content.title =
                                                            Some(title.to_string());
                                                    }
                                                    if let Some(description) =
                                                        current_schema.value_schema.description()
                                                    {
                                                        hover_value_content.description =
                                                            Some(description.to_string());
                                                    }
                                                }

                                                if !required
                                                    && hover_value_content
                                                        .accessors
                                                        .last()
                                                        .map(|accessor| accessor.is_key())
                                                        .unwrap_or_default()
                                                {
                                                    if let Some(constraints) =
                                                        &mut hover_value_content.constraints
                                                    {
                                                        constraints.key_patterns = key_patterns;
                                                    }
                                                    hover_value_content.value_type.set_nullable();
                                                }
                                            }
                                        }
                                        return hover_content;
                                    }

                                    let mut hover_content = value
                                        .get_hover_content(
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

                                    if let Some(HoverContent::Value(hover_value_content)) =
                                        hover_content.as_mut()
                                    {
                                        if keys.len() == 1
                                            && !required
                                            && hover_value_content
                                                .accessors
                                                .last()
                                                .map(|accessor| accessor.is_key())
                                                .unwrap_or_default()
                                        {
                                            if let Some(constraints) =
                                                &mut hover_value_content.constraints
                                            {
                                                constraints.key_patterns = key_patterns;
                                            }
                                            hover_value_content.value_type.set_nullable();
                                        }
                                    }

                                    return hover_content;
                                }
                                if let Some(pattern_properties) = &table_schema.pattern_properties {
                                    for (
                                        property_key,
                                        PropertySchema {
                                            property_schema, ..
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
                                                    let mut hover_content = value
                                                        .get_hover_content(
                                                            position,
                                                            &keys[1..],
                                                            &accessors
                                                                .iter()
                                                                .cloned()
                                                                .chain(std::iter::once(accessor))
                                                                .collect_vec(),
                                                            Some(&current_schema),
                                                            schema_context,
                                                        )
                                                        .await;

                                                    if let Some(HoverContent::Value(
                                                        hover_value_content,
                                                    )) = hover_content.as_mut()
                                                    {
                                                        if keys.len() == 1 {
                                                            // Check if cursor is not on the value
                                                            if !value.contains(position) {
                                                                // When cursor is on key or equals sign,
                                                                // use the property's title and description
                                                                if let Some(title) = current_schema
                                                                    .value_schema
                                                                    .title()
                                                                {
                                                                    hover_value_content.title =
                                                                        Some(title.to_string());
                                                                }
                                                                if let Some(description) =
                                                                    current_schema
                                                                        .value_schema
                                                                        .description()
                                                                {
                                                                    hover_value_content
                                                                        .description = Some(
                                                                        description.to_string(),
                                                                    );
                                                                }
                                                            }

                                                            if hover_value_content
                                                                .accessors
                                                                .last()
                                                                .map(|accessor| accessor.is_key())
                                                                .unwrap_or_default()
                                                            {
                                                                if let Some(constraints) =
                                                                    &mut hover_value_content
                                                                        .constraints
                                                                {
                                                                    constraints.key_patterns =
                                                                        key_patterns;
                                                                }
                                                                hover_value_content
                                                                    .value_type
                                                                    .set_nullable();
                                                            }
                                                        }
                                                    }
                                                    return hover_content;
                                                }

                                                let mut hover_content = value
                                                    .get_hover_content(
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

                                                if let Some(HoverContent::Value(
                                                    hover_value_content,
                                                )) = hover_content.as_mut()
                                                {
                                                    if keys.len() == 1
                                                        && hover_value_content
                                                            .accessors
                                                            .last()
                                                            .map(|accessor| accessor.is_key())
                                                            .unwrap_or_default()
                                                    {
                                                        if let Some(constraints) =
                                                            &mut hover_value_content.constraints
                                                        {
                                                            constraints.key_patterns = key_patterns;
                                                        }
                                                        hover_value_content
                                                            .value_type
                                                            .set_nullable();
                                                    }
                                                }
                                                return hover_content;
                                            }
                                        } else {
                                            tracing::warn!(
                                                "Invalid regex pattern property: {}",
                                                property_key
                                            );
                                        };
                                    }
                                }

                                if let Some((_, referable_additional_property_schema)) =
                                    &table_schema.additional_property_schema
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
                                        let mut hover_content = value
                                            .get_hover_content(
                                                position,
                                                &keys[1..],
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor.clone()))
                                                    .collect_vec(),
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await;

                                        if let Some(HoverContent::Value(hover_value_content)) =
                                            hover_content.as_mut()
                                        {
                                            if keys.len() == 1 {
                                                // Check if cursor is not on the value
                                                let cursor_on_value = value.contains(position);

                                                if !cursor_on_value {
                                                    // When cursor is on key or equals sign,
                                                    // use the property's title and description
                                                    if let Some(title) =
                                                        current_schema.value_schema.title()
                                                    {
                                                        hover_value_content.title =
                                                            Some(title.to_string());
                                                    }
                                                    if let Some(description) =
                                                        current_schema.value_schema.description()
                                                    {
                                                        hover_value_content.description =
                                                            Some(description.to_string());
                                                    }
                                                }

                                                if hover_value_content
                                                    .accessors
                                                    .last()
                                                    .map(|accessor| accessor.is_key())
                                                    .unwrap_or_default()
                                                {
                                                    hover_value_content.value_type.set_nullable();
                                                }
                                            }
                                        }
                                        return hover_content;
                                    }
                                }

                                value
                                    .get_hover_content(
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
                                    .await
                            } else {
                                None
                            }
                        } else {
                            let mut hover_content = table_schema
                                .get_hover_content(
                                    position,
                                    keys,
                                    accessors,
                                    Some(current_schema),
                                    schema_context,
                                )
                                .await;

                            if let Some(HoverContent::Value(hover_value_content)) =
                                hover_content.as_mut()
                            {
                                hover_value_content.range = Some(self.range());
                            }

                            hover_content
                        }
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        get_one_of_hover_content(
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
                        get_any_of_hover_content(
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
                        get_all_of_hover_content(
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
                    _ => None,
                }
            } else {
                if let Some(key) = keys.first() {
                    if let Some(value) = self.get(key) {
                        let accessor = Accessor::Key(key.value.clone());

                        return value
                            .get_hover_content(
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
                Some(HoverContent::Value(HoverValueContent {
                    title: None,
                    description: None,
                    accessors: Accessors::from(accessors.to_vec()),
                    value_type: ValueType::Table,
                    constraints: None,
                    schema_uri: None,
                    range: Some(self.range()),
                }))
            }
        }
        .boxed()
    }
}

impl GetHoverContent for TableSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            Some(HoverContent::Value(HoverValueContent {
                title: self.title.clone(),
                description: self.description.clone(),
                accessors: Accessors::from(accessors.to_vec()),
                value_type: ValueType::Table,
                constraints: Some(ValueConstraints {
                    enumerate: build_enumerate_values(
                        &self.const_value,
                        &self.enumerate,
                        |value| Some(value.into()),
                    ),
                    default: self.default.as_ref().map(|default| default.into()),
                    examples: self
                        .examples
                        .as_ref()
                        .map(|examples| examples.iter().map(|example| example.into()).collect()),
                    required_keys: self.required.clone(),
                    max_keys: self.max_properties,
                    min_keys: self.min_properties,
                    // NOTE: key_patterns are output for keys, not this tables.
                    key_patterns: None,
                    additional_keys: self.additional_properties(),
                    pattern_keys: self.pattern_properties.is_some(),
                    keys_order: self.keys_order.clone(),
                    array_values_order_by: self.array_values_order_by.clone(),
                    ..Default::default()
                }),
                schema_uri: current_schema.map(|schema| schema.schema_uri.as_ref().clone()),
                range: None,
            }))
        }
        .boxed()
    }
}
