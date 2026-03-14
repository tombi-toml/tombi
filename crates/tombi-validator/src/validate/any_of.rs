use std::fmt::Debug;

use tombi_ast::TombiValueCommentDirective;
use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::CurrentSchema;

use super::Validate;
use crate::validate::{if_then_else::validate_if_then_else, not_schema::validate_not};
use crate::validate::{validate_deprecated, validate_resolved_schema};

pub fn validate_any_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    any_of_schema: &'a tombi_schema_store::AnyOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_directives: Option<&'a [TombiValueCommentDirective]>,
    common_rules: Option<&'a CommonLintRules>,
) -> BoxFuture<'b, Result<crate::EvaluatedLocations, crate::Error>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    log::trace!("value = {:?}", value);
    log::trace!("any_of_schema = {:?}", any_of_schema);

    async move {
        let mut total_diagnostics = vec![];

        if let Some(not_schema) = any_of_schema.not.as_ref()
            && let Err(error) = validate_not(
                value,
                accessors,
                not_schema,
                current_schema,
                schema_context,
                comment_directives.map(|directives| directives.iter()),
                common_rules,
            )
            .await
        {
            total_diagnostics.extend(error.diagnostics);
        }

        if let Some(if_then_else_schema) = any_of_schema.if_then_else.as_ref()
            && let Err(error) = validate_if_then_else(
                value,
                accessors,
                if_then_else_schema,
                current_schema,
                schema_context,
            )
            .await
        {
            total_diagnostics.extend(error.diagnostics);
        }

        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            &any_of_schema.schemas,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
        )
        .await
        else {
            if total_diagnostics.is_empty() {
                return Ok(crate::EvaluatedLocations::new());
            } else {
                return Err(total_diagnostics.into());
            }
        };

        let mut total_error = crate::Error::new();

        for resolved_schema in &resolved_schemas {
            let Some(result) = validate_resolved_schema(
                value,
                accessors,
                resolved_schema,
                schema_context,
                comment_directives,
                common_rules,
            )
            .await
            else {
                continue;
            };

            match result {
                Ok(result) => {
                    if let Err(error) = validate_deprecated(
                        any_of_schema.deprecated,
                        accessors,
                        value,
                        comment_directives,
                        common_rules,
                    ) {
                        total_diagnostics.extend(error.diagnostics);
                    }
                    if total_diagnostics.is_empty() {
                        return Ok(result);
                    }
                    return Err(crate::Error {
                        score: crate::error::TYPE_MATCHED_SCORE,
                        diagnostics: total_diagnostics,
                        evaluated_locations: result,
                    });
                }
                Err(error) => {
                    total_error.combine(error);
                }
            }
        }

        if total_error.diagnostics.is_empty() && total_diagnostics.is_empty() {
            Ok(crate::EvaluatedLocations::new())
        } else {
            total_error.prepend_diagnostics(total_diagnostics);
            Err(total_error)
        }
    }
    .boxed()
}
