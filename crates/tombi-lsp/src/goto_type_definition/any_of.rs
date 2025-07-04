use std::borrow::Cow;

use itertools::Itertools;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUrl};

use super::{GetTypeDefinition, TypeDefinition};

pub fn get_any_of_type_definition<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    any_of_schema: &'a tombi_schema_store::AnyOfSchema,
    schema_url: &'a SchemaUrl,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> tombi_future::BoxFuture<'b, Option<TypeDefinition>>
where
    T: GetTypeDefinition + tombi_document_tree::ValueImpl + tombi_validator::Validate + Sync + Send,
{
    async move {
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
                    .is_ok()
                {
                    return Some(type_definition);
                }
            }
        }

        let mut schema_url = schema_url.clone();
        schema_url.set_fragment(Some(&format!("L{}", any_of_schema.range.start.line + 1)));

        Some(TypeDefinition {
            schema_url,
            schema_accessors: accessors.iter().map(Into::into).collect_vec(),
            range: tombi_text::Range::default(),
        })
    }
    .boxed()
}

impl GetTypeDefinition for tombi_schema_store::AnyOfSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<TypeDefinition>> {
        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            let mut schema_url = current_schema.schema_url.as_ref().to_owned();
            schema_url.set_fragment(Some(&format!("L{}", self.range.start.line + 1)));

            Some(TypeDefinition {
                schema_url,
                schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                range: tombi_text::Range::default(),
            })
        }
        .boxed()
    }
}
