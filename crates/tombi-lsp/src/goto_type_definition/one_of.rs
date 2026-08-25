use std::borrow::Cow;

use itertools::Itertools;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUri};

use super::{GetTypeDefinition, TypeDefinition, schema_type_definition};

pub fn get_one_of_type_definition<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree_syntax::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    one_of_schema: &'a tombi_schema_store::OneOfSchema,
    schema_uri: &'a SchemaUri,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    strict: Option<tombi_schema_type::BoolDefaultTrue>,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>>
where
    T: GetTypeDefinition
        + tombi_document_tree_syntax::ValueImpl
        + tombi_validator::Validate
        + Sync
        + Send
        + std::fmt::Debug,
{
    log::trace!("value: {:?}", value);
    log::trace!("keys: {:?}", keys);
    log::trace!("accessors: {:?}", accessors);
    log::trace!("one_of_schema: {:?}", one_of_schema);
    log::trace!("schema_uri: {:?}", schema_uri);

    async move {
        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            &one_of_schema.schemas,
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            strict,
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
        )
        .await
        else {
            return Vec::new();
        };
        let evaluation = tombi_validator::evaluate_applicator(
            tombi_validator::Applicator::OneOf,
            value,
            accessors,
            &resolved_schemas,
            schema_context,
        )
        .await;
        let applicable_count = evaluation.applicable_count();
        let is_property_key = keys.first().is_some_and(|key| {
            tombi_document_tree_syntax::ValueImpl::range(key).contains(position)
        });
        let mut result = Vec::new();

        for (resolved_schema, branch) in resolved_schemas.iter().zip(&evaluation.branches) {
            if !(branch.is_applicable() || applicable_count == 0 && is_property_key) {
                continue;
            }
            let projected_schema = crate::schema_resolver::project_schema_for_concrete_value(
                value,
                resolved_schema,
                schema_context,
            );
            let navigation_schema = projected_schema.as_ref().unwrap_or(resolved_schema);

            let type_definitions = value
                .get_type_definition(
                    position,
                    keys,
                    accessors,
                    Some(navigation_schema),
                    schema_context,
                )
                .await;
            result.extend(type_definitions);
        }

        if !result.is_empty() {
            return result;
        }

        let mut schema_uri = schema_uri.clone();
        schema_uri.set_fragment(Some(&format!("L{}", one_of_schema.range.start.line + 1)));

        vec![TypeDefinition {
            schema_uri,
            schema_accessors: accessors.iter().map(Into::into).collect_vec(),
            range: tombi_text::Range::default(),
        }]
    }
    .boxed()
}

impl GetTypeDefinition for tombi_schema_store::OneOfSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            vec![schema_type_definition(
                current_schema.schema_uri.as_ref(),
                accessors,
                self.range,
            )]
        }
        .boxed()
    }
}
