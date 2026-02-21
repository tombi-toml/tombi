use std::borrow::Cow;

use itertools::Itertools;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUri};

use super::{GetTypeDefinition, TypeDefinition, schema_type_definition};

pub fn get_any_of_type_definition<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    any_of_schema: &'a tombi_schema_store::AnyOfSchema,
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
    log::trace!("any_of_schema: {:?}", any_of_schema);
    log::trace!("schema_uri: {:?}", schema_uri);

    async move {
        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            &any_of_schema.schemas,
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            schema_context.store,
            accessors,
        )
        .await
        else {
            return None;
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
                && value
                    .validate(accessors, Some(resolved_schema), schema_context)
                    .await
                    .is_ok()
            {
                return Some(type_definition);
            }
        }

        let mut schema_uri = schema_uri.clone();
        schema_uri.set_fragment(Some(&format!("L{}", any_of_schema.range.start.line + 1)));

        Some(TypeDefinition {
            schema_uri,
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

            Some(schema_type_definition(
                current_schema.schema_uri.as_ref(),
                accessors,
                self.range,
            ))
        }
        .boxed()
    }
}
