use std::borrow::Cow;
use tombi_future::Boxable;

use itertools::Itertools;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUri};

use super::{GetTypeDefinition, TypeDefinition};

pub fn get_all_of_type_definition<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    schema_uri: &'a SchemaUri,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> tombi_future::BoxFuture<'b, Option<TypeDefinition>>
where
    T: GetTypeDefinition
        + tombi_document_tree::ValueImpl
        + tombi_validator::Validate
        + Sync
        + Send
        + std::fmt::Debug,
{
    log::trace!("value: {:?}", value);
    log::trace!("keys: {:?}", keys);
    log::trace!("accessors: {:?}", accessors);
    log::trace!("all_of_schema: {:?}", all_of_schema);
    log::trace!("schema_uri: {:?}", schema_uri);

    async move {
        let mut all_of_type_definition = None;

        let Ok(mut schemas_guard) = all_of_schema.schemas.try_write() else {
            log::warn!("Circular JSON Schema reference detected in get_all_of_goto_type_definition_response, skipping");
            return None;
        };
        let resolved_schemas = {
            let mut resolved = Vec::with_capacity(schemas_guard.len());
            for referable_schema in schemas_guard.iter_mut() {
                if let Ok(Some(current_schema)) = referable_schema
                    .resolve(
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        schema_context.store,
                    )
                    .await
                {
                    resolved.push(current_schema.into_owned());
                }
            }
            resolved
        };

        for resolved_schema in &resolved_schemas {
            if let Some(type_definition) = value
                .get_type_definition(
                    position,
                    keys,
                    accessors,
                    Some(resolved_schema),
                    schema_context,
                )
                .await
            {
                if value
                    .validate(accessors, Some(resolved_schema), schema_context)
                    .await
                    .is_err()
                {
                    return Some(TypeDefinition {
                        schema_uri: schema_uri.clone(),
                        schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                        range: tombi_text::Range::default(),
                    });
                }
                all_of_type_definition = Some(type_definition);
            }
        }

        drop(schemas_guard);

        all_of_type_definition
    }
    .boxed()
}

impl GetTypeDefinition for tombi_schema_store::AllOfSchema {
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

            let mut schema_uri = current_schema.schema_uri.as_ref().to_owned();
            schema_uri.set_fragment(Some(&format!("L{}", self.range.start.line + 1)));

            Some(TypeDefinition {
                schema_uri,
                schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                range: tombi_text::Range::default(),
            })
        }
        .boxed()
    }
}
