use std::borrow::Cow;

use futures::{future::BoxFuture, FutureExt};
use itertools::Itertools;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUrl};

use super::{GetTypeDefinition, TypeDefinition};

pub fn get_all_of_type_definition<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    schema_url: &'a SchemaUrl,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> BoxFuture<'b, Option<TypeDefinition>>
where
    T: GetTypeDefinition + tombi_document_tree::ValueImpl + tombi_validator::Validate + Sync + Send,
{
    async move {
        let mut all_of_type_definition = None;

        for referable_schema in all_of_schema.schemas.write().await.iter_mut() {
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

            if let Some(type_definition) = value
                .get_type_definition(
                    position,
                    keys,
                    accessors,
                    Some(&current_schema),
                    schema_context,
                )
                .await
            {
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
                    .is_err()
                {
                    return Some(TypeDefinition {
                        schema_url: schema_url.clone(),
                        range: tombi_text::Range::default(),
                    });
                }
                all_of_type_definition = Some(type_definition);
            }
        }

        all_of_type_definition
    }
    .boxed()
}

impl GetTypeDefinition for tombi_schema_store::AllOfSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        _accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<TypeDefinition>> {
        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            Some(TypeDefinition {
                schema_url: current_schema.schema_url.as_ref().to_owned(),
                range: tombi_text::Range::default(),
            })
        }
        .boxed()
    }
}
