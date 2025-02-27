use std::borrow::Cow;

use futures::{future::BoxFuture, FutureExt};
use schema_store::{Accessor, CurrentSchema, SchemaUrl, ValueSchema};

use super::{GetHoverContent, HoverContent};

pub fn get_one_of_hover_content<'a: 'b, 'b, T>(
    value: &'a T,
    position: text::Position,
    keys: &'a [document_tree::Key],
    accessors: &'a [schema_store::Accessor],
    schema_url: &'a SchemaUrl,
    one_of_schema: &'a schema_store::OneOfSchema,
    definitions: &'a schema_store::SchemaDefinitions,
    schema_context: &'a schema_store::SchemaContext,
) -> BoxFuture<'b, Option<HoverContent>>
where
    T: GetHoverContent + document_tree::ValueImpl + Sync + Send,
{
    async move {
        let mut one_hover_contents = ahash::AHashSet::new();
        let mut value_type_set = indexmap::IndexSet::new();

        for referable_schema in one_of_schema.schemas.write().await.iter_mut() {
            let Ok(CurrentSchema { value_schema, .. }) = referable_schema
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
            schema_store::ValueType::OneOf(value_type_set.into_iter().collect())
        };

        for referable_schema in one_of_schema.schemas.read().await.iter() {
            let Some(value_schema) = referable_schema.resolved() else {
                continue;
            };
            if let Some(mut hover_content) = value
                .get_hover_content(
                    position,
                    keys,
                    accessors,
                    Some(schema_url),
                    Some(value_schema),
                    Some(definitions),
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

                if value_schema.value_type().await == schema_store::ValueType::Array
                    && hover_content.value_type != schema_store::ValueType::Array
                {
                    return Some(hover_content);
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

impl GetHoverContent for schema_store::OneOfSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: text::Position,
        _keys: &'a [document_tree::Key],
        accessors: &'a [Accessor],
        schema_url: Option<&'a SchemaUrl>,
        _value_schema: Option<&'a ValueSchema>,
        definitions: Option<&'a schema_store::SchemaDefinitions>,
        schema_context: &'a schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<HoverContent>> {
        async move {
            let (Some(schema_url), Some(definitions)) = (schema_url, definitions) else {
                unreachable!("schema must be provided");
            };

            let mut title_description_set = ahash::AHashSet::new();
            let mut value_type_set = indexmap::IndexSet::new();

            for referable_schema in self.schemas.write().await.iter_mut() {
                let Ok(CurrentSchema { value_schema, .. }) = referable_schema
                    .resolve(
                        Cow::Borrowed(&schema_url),
                        Cow::Borrowed(definitions),
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
                schema_store::ValueType::OneOf(value_type_set.into_iter().collect())
            };

            Some(HoverContent {
                title,
                description,
                accessors: schema_store::Accessors::new(accessors.to_vec()),
                value_type,
                constraints: None,
                schema_url: Some(schema_url.to_owned()),
                range: None,
            })
        }
        .boxed()
    }
}
