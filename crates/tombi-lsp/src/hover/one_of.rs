use std::borrow::Cow;

use futures::{future::BoxFuture, FutureExt};
use itertools::Itertools;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUrl};

use super::{
    constraints::ValueConstraints, display_value::DisplayValue, GetHoverContent, HoverContent,
};

pub fn get_one_of_hover_content<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    one_of_schema: &'a tombi_schema_store::OneOfSchema,
    schema_url: &'a SchemaUrl,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> BoxFuture<'b, Option<HoverContent>>
where
    T: GetHoverContent + tombi_document_tree::ValueImpl + tombi_validator::Validate + Sync + Send,
{
    async move {
        let mut one_hover_contents = ahash::AHashSet::new();
        let mut valid_hover_contents = ahash::AHashSet::new();
        let mut value_type_set = indexmap::IndexSet::new();
        let mut enumerate_values = Vec::new();
        let default = one_of_schema
            .default
            .as_ref()
            .and_then(|default| DisplayValue::try_from(default).ok());

        for referable_schema in one_of_schema.schemas.write().await.iter_mut() {
            let Ok(Some(CurrentSchema { value_schema, .. })) = referable_schema
                .resolve(
                    Cow::Borrowed(schema_url),
                    Cow::Borrowed(definitions),
                    schema_context.store,
                )
                .await
            else {
                continue;
            };
            value_type_set.insert(value_schema.value_type().await);
        }

        let value_type = if value_type_set.len() == 1 {
            value_type_set.into_iter().next().unwrap()
        } else {
            tombi_schema_store::ValueType::OneOf(value_type_set.into_iter().collect())
        };

        for referable_schema in one_of_schema.schemas.write().await.iter_mut() {
            let Ok(Some(current_schema)) = referable_schema
                .resolve(
                    Cow::Borrowed(schema_url),
                    Cow::Borrowed(definitions),
                    schema_context.store,
                )
                .await
            else {
                continue;
            };

            if let Some(mut hover_content) = value
                .get_hover_content(
                    position,
                    keys,
                    accessors,
                    Some(&current_schema),
                    schema_context,
                )
                .await
            {
                if hover_content.title.is_none() && hover_content.description.is_none() {
                    if let Some(title) = &one_of_schema.title {
                        hover_content.title = Some(title.clone());
                    }
                    if let Some(description) = &one_of_schema.description {
                        hover_content.description = Some(description.clone());
                    }
                }

                if keys.is_empty() && accessors == hover_content.accessors.as_ref() {
                    hover_content.value_type = value_type.clone();
                }

                if current_schema.value_schema.value_type().await
                    == tombi_schema_store::ValueType::Array
                    && hover_content.value_type != tombi_schema_store::ValueType::Array
                {
                    return Some(hover_content);
                }

                if let Some(enumerate) = &hover_content
                    .constraints
                    .as_ref()
                    .and_then(|constraints| constraints.enumerate.as_ref())
                {
                    enumerate_values.extend(enumerate.iter().map(Clone::clone));
                }

                match value
                    .validate(
                        &accessors
                            .iter()
                            .map(|accessor| accessor.into())
                            .collect_vec(),
                        Some(&current_schema),
                        schema_context,
                    )
                    .await
                {
                    Ok(()) => {
                        valid_hover_contents.insert(hover_content.clone());
                    }
                    Err(errors)
                        if errors
                            .iter()
                            .all(|error| error.level() == tombi_diagnostic::Level::WARNING) =>
                    {
                        valid_hover_contents.insert(hover_content.clone());
                    }
                    _ => {}
                }

                one_hover_contents.insert(hover_content);
            }
        }

        if one_hover_contents.len() == 1 {
            one_hover_contents
                .into_iter()
                .next()
                .map(|mut hover_content| {
                    if hover_content.title.is_none() && hover_content.description.is_none() {
                        if let Some(title) = &one_of_schema.title {
                            hover_content.title = Some(title.clone());
                        }
                        if let Some(description) = &one_of_schema.description {
                            hover_content.description = Some(description.clone());
                        }
                    }

                    hover_content
                })
        } else if valid_hover_contents.len() == 1 {
            valid_hover_contents
                .into_iter()
                .next()
                .map(|mut hover_content| {
                    if hover_content.title.is_none() && hover_content.description.is_none() {
                        if let Some(title) = &one_of_schema.title {
                            hover_content.title = Some(title.clone());
                        }
                        if let Some(description) = &one_of_schema.description {
                            hover_content.description = Some(description.clone());
                        }
                    }

                    hover_content
                })
        } else {
            Some(HoverContent {
                title: None,
                description: None,
                accessors: tombi_schema_store::Accessors::new(accessors.to_vec()),
                value_type: value.value_type().into(),
                constraints: None,
                schema_url: Some(schema_url.to_owned()),
                range: None,
            })
        }
        .map(|mut hover_content| {
            if let Some(default) = default {
                if let Some(constraints) = hover_content.constraints.as_mut() {
                    if constraints.default.is_none() {
                        constraints.default = Some(default);
                    }
                } else {
                    hover_content.constraints = Some(ValueConstraints {
                        default: Some(default),
                        ..Default::default()
                    });
                }
            }

            if !enumerate_values.is_empty() {
                if let Some(constraints) = hover_content.constraints.as_mut() {
                    constraints.enumerate = Some(enumerate_values);
                } else {
                    hover_content.constraints = Some(ValueConstraints {
                        enumerate: Some(enumerate_values),
                        ..Default::default()
                    });
                }
            }

            hover_content
        })
    }
    .boxed()
}

impl GetHoverContent for tombi_schema_store::OneOfSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<HoverContent>> {
        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            let mut title_description_set = ahash::AHashSet::new();
            let mut value_type_set = indexmap::IndexSet::new();

            for referable_schema in self.schemas.write().await.iter_mut() {
                let Ok(Some(CurrentSchema { value_schema, .. })) = referable_schema
                    .resolve(
                        current_schema.schema_url.clone(),
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
                tombi_schema_store::ValueType::OneOf(value_type_set.into_iter().collect())
            };

            Some(HoverContent {
                title,
                description,
                accessors: tombi_schema_store::Accessors::new(accessors.to_vec()),
                value_type,
                constraints: None,
                schema_url: Some(current_schema.schema_url.as_ref().to_owned()),
                range: None,
            })
        }
        .boxed()
    }
}
