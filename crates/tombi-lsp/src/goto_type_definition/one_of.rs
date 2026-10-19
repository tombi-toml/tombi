use std::borrow::Cow;

use itertools::Itertools;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema};

use super::{GetTypeDefinition, TypeDefinition, schema_type_definition};

pub fn get_one_of_type_definition<'a: 'b, 'b, T>(
    value: &'a T,
    cursor: crate::CursorPosition<'a>,
    keys: &'a [tombi_document_tree_syntax::Key<'_>],
    accessors: &'a [tombi_schema_store::Accessor],
    one_of_schema: &'a tombi_schema_store::OneOfSchema,
    current_schema: &'a CurrentSchema<'a>,
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
    let offset = cursor.offset();
    log::trace!("value: {:?}", value);
    log::trace!("keys: {:?}", keys);
    log::trace!("accessors: {:?}", accessors);
    log::trace!("one_of_schema: {:?}", one_of_schema);
    log::trace!("schema_base_uri: {:?}", current_schema.schema_base_uri);

    async move {
        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas_in_scope(
            &one_of_schema.schemas,
            Cow::Borrowed(current_schema.schema_base_uri.as_ref()),
            Cow::Borrowed(current_schema.definitions.as_ref()),
            current_schema.strict,
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
            Some(&current_schema.dynamic_scope),
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
            tombi_document_tree_syntax::ValueImpl::span(key).contains_inclusive(offset)
        });
        let mut result = Vec::new();

        for (resolved_schema, branch) in resolved_schemas.iter().zip(&evaluation.branches) {
            if !(branch.is_applicable() || applicable_count == 0 && is_property_key) {
                continue;
            }
            let projected_schema = crate::schema_resolver::project_composite_branch_schema(
                value,
                resolved_schema,
                schema_context,
            );
            let navigation_schema = projected_schema.as_ref().unwrap_or(resolved_schema);

            let type_definitions = value
                .get_type_definition(
                    cursor,
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

        let mut schema_base_uri = current_schema.schema_base_uri.as_ref().clone();
        schema_base_uri.set_fragment(Some(&super::schema_line_fragment(
            &current_schema.line_index,
            one_of_schema.span,
        )));

        vec![TypeDefinition {
            schema_base_uri,
            schema_accessors: accessors.iter().map(Into::into).collect_vec(),
            span: None,
        }]
    }
    .boxed()
}

impl GetTypeDefinition for tombi_schema_store::OneOfSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _cursor: crate::CursorPosition<'a>,
        _keys: &'a [tombi_document_tree_syntax::Key<'_>],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            vec![schema_type_definition(current_schema, accessors, self.span)]
        }
        .boxed()
    }
}
