use std::borrow::Cow;

use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, Accessors, ArraySchema, CurrentSchema, DocumentSchema, ValueSchema, ValueType,
};

use crate::hover::{
    all_of::get_all_of_hover_content,
    any_of::get_any_of_hover_content,
    constraints::{build_enumerate_values, ValueConstraints},
    display_value::DisplayValue,
    one_of::get_one_of_hover_content,
    GetHoverContent, HoverContent,
};

impl GetHoverContent for tombi_document_tree::Array {
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
            if let Some(Ok(DocumentSchema {
                value_schema,
                schema_url,
                definitions,
                ..
            })) = schema_context
                .get_subschema(accessors, current_schema)
                .await
            {
                let current_schema = value_schema.map(|value_schema| CurrentSchema {
                    value_schema: Cow::Owned(value_schema),
                    schema_url: Cow::Owned(schema_url),
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
                    ValueSchema::Array(array_schema) => {
                        for (index, value) in self.values().iter().enumerate() {
                            if value.range().contains(position) {
                                let accessor = Accessor::Index(index);

                                if let Some(items) = &array_schema.items {
                                    let mut referable_schema = items.write().await;
                                    if let Ok(Some(current_schema)) = referable_schema
                                        .resolve(
                                            current_schema.schema_url.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        let mut hover_content = value
                                            .get_hover_content(
                                                position,
                                                keys,
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor.clone()))
                                                    .collect::<Vec<_>>(),
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await?;

                                        if keys.is_empty()
                                            && self.kind()
                                                == tombi_document_tree::ArrayKind::ArrayOfTable
                                        {
                                            if let Some(constraints) =
                                                &mut hover_content.constraints
                                            {
                                                constraints.min_items = array_schema.min_items;
                                                constraints.max_items = array_schema.max_items;
                                                constraints.unique_items =
                                                    array_schema.unique_items;
                                            }
                                        }

                                        if hover_content.title.is_none()
                                            && hover_content.description.is_none()
                                        {
                                            if let Some(title) = &array_schema.title {
                                                hover_content.title = Some(title.clone());
                                            }
                                            if let Some(description) = &array_schema.description {
                                                hover_content.description =
                                                    Some(description.clone());
                                            }
                                        }
                                        return Some(hover_content);
                                    }
                                }

                                return value
                                    .get_hover_content(
                                        position,
                                        keys,
                                        &accessors
                                            .iter()
                                            .cloned()
                                            .chain(std::iter::once(accessor))
                                            .collect::<Vec<_>>(),
                                        None,
                                        schema_context,
                                    )
                                    .await;
                            }
                        }
                        return array_schema
                            .get_hover_content(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await
                            .map(|mut hover_content| {
                                hover_content.range = Some(self.range());
                                hover_content
                            });
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        return get_one_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            &current_schema.schema_url,
                            &current_schema.definitions,
                            schema_context,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        return get_any_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            &current_schema.schema_url,
                            &current_schema.definitions,
                            schema_context,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        return get_all_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            &current_schema.schema_url,
                            &current_schema.definitions,
                            schema_context,
                        )
                        .await
                    }
                    ValueSchema::Null => {
                        for (index, value) in self.values().iter().enumerate() {
                            if value.range().contains(position) {
                                let accessor = Accessor::Index(index);
                                return value
                                    .get_hover_content(
                                        position,
                                        keys,
                                        &accessors
                                            .iter()
                                            .cloned()
                                            .chain(std::iter::once(accessor))
                                            .collect::<Vec<_>>(),
                                        Some(current_schema),
                                        schema_context,
                                    )
                                    .await;
                            }
                        }

                        return Some(HoverContent {
                            title: None,
                            description: None,
                            accessors: Accessors::new(accessors.to_vec()),
                            value_type: ValueType::Array,
                            constraints: None,
                            schema_url: None,
                            range: Some(self.range()),
                        });
                    }
                    _ => {}
                }
            }

            for (index, value) in self.values().iter().enumerate() {
                if value.range().contains(position) {
                    let accessor = Accessor::Index(index);
                    return value
                        .get_hover_content(
                            position,
                            keys,
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(accessor))
                                .collect::<Vec<_>>(),
                            None,
                            schema_context,
                        )
                        .await;
                }
            }
            Some(HoverContent {
                title: None,
                description: None,
                accessors: Accessors::new(accessors.to_vec()),
                value_type: ValueType::Array,
                constraints: None,
                schema_url: None,
                range: Some(self.range()),
            })
        }
        .boxed()
    }
}

impl GetHoverContent for ArraySchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            Some(HoverContent {
                title: self.title.clone(),
                description: self.description.clone(),
                accessors: Accessors::new(accessors.to_vec()),
                value_type: ValueType::Array,
                constraints: Some(ValueConstraints {
                    enumerate: build_enumerate_values(
                        &self.const_value,
                        &self.enumerate,
                        |value| DisplayValue::try_from(value).ok(),
                    ),
                    default: self
                        .default
                        .as_ref()
                        .and_then(|default| DisplayValue::try_from(default).ok()),
                    examples: self.examples.as_ref().map(|examples| {
                        examples
                            .iter()
                            .filter_map(|example| DisplayValue::try_from(example).ok())
                            .collect()
                    }),
                    min_items: self.min_items,
                    max_items: self.max_items,
                    unique_items: self.unique_items,
                    values_order: self.values_order.clone(),
                    ..Default::default()
                }),
                schema_url: current_schema.map(|cs| cs.schema_url.as_ref().clone()),
                range: None,
            })
        }
        .boxed()
    }
}
