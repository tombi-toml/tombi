use std::borrow::Cow;

use futures::{future::BoxFuture, FutureExt};
use schema_store::{Accessor, CurrentSchema, SchemaContext, SchemaUrl, ValueSchema};

use super::{GetHoverContent, HoverContent};

pub fn get_all_of_hover_content<'a: 'b, 'b, T>(
    value: &'a T,
    position: text::Position,
    keys: &'a [document_tree::Key],
    accessors: &'a [schema_store::Accessor],
    schema_url: &'a SchemaUrl,
    all_of_schema: &'a schema_store::AllOfSchema,
    definitions: &'a schema_store::SchemaDefinitions,
    schema_context: &'a SchemaContext,
) -> BoxFuture<'b, Option<HoverContent>>
where
    T: GetHoverContent + Sync + Send,
{
    async move {
        let mut title_description_set = ahash::AHashSet::new();
        let mut value_type_set = indexmap::IndexSet::new();
        let mut schema = None;
        for referable_schema in all_of_schema.schemas.write().await.iter_mut() {
            let Ok(CurrentSchema {
                value_schema,
                schema_url,
                definitions,
            }) = referable_schema
                .resolve(
                    Cow::Borrowed(schema_url),
                    Cow::Borrowed(definitions),
                    schema_context.store,
                )
                .await
            else {
                return None;
            };

            if let Some(hover_content) = value
                .get_hover_content(
                    position,
                    keys,
                    accessors,
                    Some(&schema_url),
                    Some(value_schema),
                    Some(&definitions),
                    schema_context,
                )
                .await
            {
                if hover_content.title.is_some() || hover_content.description.is_some() {
                    title_description_set.insert((
                        hover_content.title.clone(),
                        hover_content.description.clone(),
                    ));
                    value_type_set.insert(hover_content.value_type);
                }

                if let Some(s) = hover_content.constraints {
                    schema = Some(s);
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
            schema_store::ValueType::AllOf(value_type_set.into_iter().collect())
        };

        Some(HoverContent {
            title,
            description,
            accessors: schema_store::Accessors::new(accessors.to_vec()),
            value_type,
            constraints: schema,
            schema_url: Some(schema_url.to_owned()),
            range: None,
        })
    }
    .boxed()
}

impl GetHoverContent for schema_store::AllOfSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: text::Position,
        _keys: &'a [document_tree::Key],
        accessors: &'a [Accessor],
        schema_url: Option<&'a SchemaUrl>,
        _value_schema: Option<&'a ValueSchema>,
        definitions: Option<&'a schema_store::SchemaDefinitions>,
        schema_context: &'a SchemaContext,
    ) -> BoxFuture<'b, Option<HoverContent>> {
        async move {
            let (Some(schema_url), Some(definitions)) = (schema_url, definitions) else {
                unreachable!("schema must be provided");
            };

            let mut title_description_set = ahash::AHashSet::new();
            let mut value_type_set = indexmap::IndexSet::new();
            let mut schemas = self.schemas.write().await;

            for referable_schema in schemas.iter_mut() {
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
                schema_store::ValueType::AllOf(value_type_set.into_iter().collect())
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
