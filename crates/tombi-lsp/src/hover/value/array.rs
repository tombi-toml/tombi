use std::borrow::Cow;

use itertools::Itertools;

use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, Accessors, ArraySchema, CurrentSchema, DocumentSchema, ValueSchema, ValueType,
};

use crate::{
    comment_directive::get_array_comment_directive_content_with_schema_uri,
    hover::{
        all_of::get_all_of_hover_content,
        any_of::get_any_of_hover_content,
        comment::get_value_comment_directive_hover_content,
        constraints::{build_enumerate_values, ValueConstraints},
        display_value::DisplayValue,
        one_of::get_one_of_hover_content,
        GetHoverContent, HoverValueContent,
    },
    HoverContent,
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
            if let Some((comment_directive_context, schema_uri)) =
                get_array_comment_directive_content_with_schema_uri(self, position, accessors)
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
                    ValueSchema::Array(array_schema) => {
                        for (index, value) in self.values().iter().enumerate() {
                            if value.contains(position) {
                                let accessor = Accessor::Index(index);

                                if let Some(items) = &array_schema.items {
                                    let mut referable_schema = items.write().await;
                                    if let Ok(Some(current_schema)) = referable_schema
                                        .resolve(
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        return match value
                                            .get_hover_content(
                                                position,
                                                keys,
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor.clone()))
                                                    .collect_vec(),
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await?
                                        {
                                            HoverContent::Value(mut hover_value_content) => {
                                                if keys.is_empty()
                                            && self.kind()
                                                == tombi_document_tree::ArrayKind::ArrayOfTable
                                        {
                                            if let Some(constraints) =
                                                &mut hover_value_content.constraints
                                            {
                                                constraints.min_items = array_schema.min_items;
                                                constraints.max_items = array_schema.max_items;
                                                constraints.unique_items =
                                                    array_schema.unique_items;
                                            }
                                        }

                                                if hover_value_content.title.is_none()
                                                    && hover_value_content.description.is_none()
                                                {
                                                    if let Some(title) = &array_schema.title {
                                                        hover_value_content.title =
                                                            Some(title.clone());
                                                    }
                                                    if let Some(description) =
                                                        &array_schema.description
                                                    {
                                                        hover_value_content.description =
                                                            Some(description.clone());
                                                    }
                                                }
                                                Some(HoverContent::Value(hover_value_content))
                                            }
                                            HoverContent::Directive(hover_content) => {
                                                Some(HoverContent::Directive(hover_content))
                                            }
                                            HoverContent::DirectiveContent(hover_content) => {
                                                Some(HoverContent::DirectiveContent(hover_content))
                                            }
                                        };
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
                                            .collect_vec(),
                                        None,
                                        schema_context,
                                    )
                                    .await;
                            }
                        }
                        let mut hover_content = array_schema
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

                        return hover_content;
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        return get_one_of_hover_content(
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
                        return get_any_of_hover_content(
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
                        return get_all_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            schema_context,
                        )
                        .await;
                    }
                    ValueSchema::Null => {
                        for (index, value) in self.values().iter().enumerate() {
                            if value.contains(position) {
                                let accessor = Accessor::Index(index);
                                return value
                                    .get_hover_content(
                                        position,
                                        keys,
                                        &accessors
                                            .iter()
                                            .cloned()
                                            .chain(std::iter::once(accessor))
                                            .collect_vec(),
                                        Some(current_schema),
                                        schema_context,
                                    )
                                    .await;
                            }
                        }

                        return Some(HoverContent::Value(HoverValueContent {
                            title: None,
                            description: None,
                            accessors: Accessors::from(accessors.to_vec()),
                            value_type: ValueType::Array,
                            constraints: None,
                            schema_uri: None,
                            range: Some(self.range()),
                        }));
                    }
                    _ => {}
                }
            }

            for (index, value) in self.values().iter().enumerate() {
                if value.contains(position) {
                    let accessor = Accessor::Index(index);
                    return value
                        .get_hover_content(
                            position,
                            keys,
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
                value_type: ValueType::Array,
                constraints: None,
                schema_uri: None,
                range: Some(self.range()),
            }))
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
            Some(HoverContent::Value(HoverValueContent {
                title: self.title.clone(),
                description: self.description.clone(),
                accessors: Accessors::from(accessors.to_vec()),
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
                schema_uri: current_schema.map(|cs| cs.schema_uri.as_ref().clone()),
                range: None,
            }))
        }
        .boxed()
    }
}
