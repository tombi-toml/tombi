use std::borrow::Cow;

use futures::{future::BoxFuture, FutureExt};
use itertools::Itertools;
use schema_store::{Accessor, CurrentSchema, SchemaContext, SchemaUrl};

use super::{GetHoverContent, HoverContent};

pub fn get_any_of_hover_content<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [schema_store::Accessor],
    any_of_schema: &'a schema_store::AnyOfSchema,
    schema_url: &'a SchemaUrl,
    definitions: &'a schema_store::SchemaDefinitions,
    schema_context: &'a SchemaContext,
) -> BoxFuture<'b, Option<HoverContent>>
where
    T: GetHoverContent + tombi_document_tree::ValueImpl + tombi_validator::Validate + Sync + Send,
{
    async move {
        let mut any_hover_contents = vec![];
        let mut valid_hover_contents = vec![];
        let mut value_type_set = indexmap::IndexSet::new();

        for referable_schema in any_of_schema.schemas.write().await.iter_mut() {
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
            schema_store::ValueType::AnyOf(value_type_set.into_iter().collect())
        };

        for referable_schema in any_of_schema.schemas.write().await.iter_mut() {
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
                    if let Some(title) = &any_of_schema.title {
                        hover_content.title = Some(title.clone());
                    }
                    if let Some(description) = &any_of_schema.description {
                        hover_content.description = Some(description.clone());
                    }
                }

                if keys.is_empty() && accessors == hover_content.accessors.as_ref() {
                    hover_content.value_type = value_type.clone();
                }

                if value
                    .validate(
                        &accessors
                            .iter()
                            .map(|accessor| accessor.into())
                            .collect_vec(),
                        Some(&current_schema),
                        schema_context,
                    )
                    .await
                    .is_ok()
                {
                    valid_hover_contents.push(hover_content.clone());
                }

                any_hover_contents.push(hover_content);
            }
        }

        if let Some(hover_content) = valid_hover_contents.into_iter().next() {
            Some(hover_content)
        } else if let Some(hover_content) = any_hover_contents.into_iter().next() {
            Some(hover_content)
        } else {
            Some(HoverContent {
                title: None,
                description: None,
                accessors: schema_store::Accessors::new(accessors.to_vec()),
                value_type: value.value_type().into(),
                constraints: None,
                schema_url: Some(schema_url.to_owned()),
                range: None,
            })
        }
    }
    .boxed()
}

impl GetHoverContent for schema_store::AnyOfSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a SchemaContext,
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

            let value_type: schema_store::ValueType = if value_type_set.len() == 1 {
                value_type_set.into_iter().next().unwrap()
            } else {
                schema_store::ValueType::AnyOf(value_type_set.into_iter().collect())
            };

            Some(HoverContent {
                title,
                description,
                accessors: schema_store::Accessors::new(accessors.to_vec()),
                value_type,
                constraints: None,
                schema_url: Some(current_schema.schema_url.as_ref().clone()),
                range: None,
            })
        }
        .boxed()
    }
}
