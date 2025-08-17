use std::borrow::Cow;

use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaContext, SchemaUri};

use crate::hover::display_value::GetEnumerate;

use super::{
    constraints::ValueConstraints, display_value::DisplayValue, GetHoverContent, HoverValueContent,
};

pub fn get_all_of_hover_content<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    schema_uri: &'a SchemaUri,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    schema_context: &'a SchemaContext,
) -> tombi_future::BoxFuture<'b, Option<HoverValueContent>>
where
    T: GetHoverContent + Sync + Send,
{
    async move {
        let mut title_description_set = ahash::AHashSet::new();
        let mut value_type_set = indexmap::IndexSet::new();
        let mut constraints = None;
        let mut enumerate_values = Vec::new();
        let default = all_of_schema
            .default
            .as_ref()
            .and_then(|default| DisplayValue::try_from(default).ok());

        for referable_schema in all_of_schema.schemas.write().await.iter_mut() {
            let Ok(Some(current_schema)) = referable_schema
                .resolve(
                    Cow::Borrowed(schema_uri),
                    Cow::Borrowed(definitions),
                    schema_context.store,
                )
                .await
            else {
                continue;
            };

            if let Some(values) = current_schema
                .value_schema
                .as_ref()
                .get_enumerate(schema_uri, definitions, schema_context)
                .await
            {
                enumerate_values.extend(values);
            }

            if let Some(hover_content) = value
                .get_hover_content(
                    position,
                    keys,
                    accessors,
                    Some(&current_schema),
                    schema_context,
                )
                .await
            {
                if hover_content.title.is_some() || hover_content.description.is_some() {
                    title_description_set.insert((
                        hover_content.title.clone(),
                        hover_content.description.clone(),
                    ));
                }
                value_type_set.insert(hover_content.value_type);

                if let Some(c) = hover_content.constraints {
                    constraints = Some(c);
                }
            }
        }

        let (mut title, mut description) = if title_description_set.len() == 1 {
            title_description_set.into_iter().next().unwrap()
        } else {
            (None, None)
        };

        if title.is_none() && description.is_none() {
            if let Some(t) = &all_of_schema.title {
                title = Some(t.clone());
            }
            if let Some(d) = &all_of_schema.description {
                description = Some(d.clone());
            }
        }

        let value_type = if value_type_set.len() == 1 {
            value_type_set.into_iter().next().unwrap()
        } else {
            tombi_schema_store::ValueType::AllOf(value_type_set.into_iter().collect())
        };

        if let Some(default) = default {
            if let Some(constraints) = constraints.as_mut() {
                if constraints.default.is_none() {
                    constraints.default = Some(default);
                }
            } else {
                constraints = Some(ValueConstraints {
                    default: Some(default),
                    ..Default::default()
                });
            }
        }

        if !enumerate_values.is_empty() {
            if let Some(constraints) = constraints.as_mut() {
                constraints.enumerate = Some(enumerate_values);
            } else {
                constraints = Some(ValueConstraints {
                    enumerate: Some(enumerate_values),
                    ..Default::default()
                });
            }
        }

        Some(HoverValueContent {
            title,
            description,
            accessors: tombi_schema_store::Accessors::new(accessors.to_vec()),
            value_type,
            constraints,
            schema_uri: Some(schema_uri.to_owned()),
            range: None,
        })
    }
    .boxed()
}

impl GetHoverContent for tombi_schema_store::AllOfSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverValueContent>> {
        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            let mut title_description_set = ahash::AHashSet::new();
            let mut value_type_set = indexmap::IndexSet::new();
            let mut schemas = self.schemas.write().await;

            for referable_schema in schemas.iter_mut() {
                let Ok(Some(CurrentSchema { value_schema, .. })) = referable_schema
                    .resolve(
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                    )
                    .await
                else {
                    return None;
                };
                if value_schema.title().is_some() || value_schema.description().is_some() {
                    title_description_set.insert((
                        value_schema.title().map(ToString::to_string),
                        value_schema.description().map(ToString::to_string),
                    ));
                }
                value_type_set.insert(value_schema.value_type().await);
            }

            let (mut title, mut description) = if title_description_set.len() == 1 {
                title_description_set.into_iter().next().unwrap()
            } else {
                (None, None)
            };

            if title.is_none() && description.is_none() {
                if let Some(t) = &self.title {
                    title = Some(t.clone());
                }
                if let Some(d) = &self.description {
                    description = Some(d.clone());
                }
            }

            let value_type: tombi_schema_store::ValueType = if value_type_set.len() == 1 {
                value_type_set.into_iter().next().unwrap()
            } else {
                tombi_schema_store::ValueType::AllOf(value_type_set.into_iter().collect())
            };

            Some(HoverValueContent {
                title,
                description,
                accessors: tombi_schema_store::Accessors::new(accessors.to_vec()),
                value_type,
                constraints: None,
                schema_uri: Some(current_schema.schema_uri.as_ref().to_owned()),
                range: None,
            })
        }
        .boxed()
    }
}
