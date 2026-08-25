use std::fmt::Debug;

use tombi_ast_syntax::TombiValueCommentDirective;
use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree_syntax::ValueImpl;
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
) -> BoxFuture<'b, Result<crate::Valid, crate::Invalid>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    log::trace!("value = {:?}", value);
    log::trace!("any_of_schema = {:?}", any_of_schema);

    async move {
        let mut total_diagnostics = vec![];
        let mut adjacent_assertion_failed = false;
        let mut base_evaluated_locations = crate::Valid::new();

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
            adjacent_assertion_failed |= error.assertion_failed;
            total_diagnostics.extend(error.diagnostics);
        }

        if let Some(if_then_else_schema) = any_of_schema.if_then_else.as_ref() {
            match validate_if_then_else(
                value,
                accessors,
                if_then_else_schema,
                current_schema,
                schema_context,
                common_rules,
            )
            .await
            {
                Ok(result) => base_evaluated_locations.merge_from(result),
                Err(error) => {
                    adjacent_assertion_failed |= error.assertion_failed;
                    if !error.assertion_failed {
                        base_evaluated_locations
                            .merge_from(error.local_evaluated_locations.clone());
                    }
                    total_diagnostics.extend(error.diagnostics);
                }
            }
        }

        let Some((resolved_schemas, resolution_errors)) =
            tombi_schema_store::resolve_and_collect_schemas_with_errors(
                &any_of_schema.schemas,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                current_schema.strict,
                schema_context.store,
                &schema_context.schema_visits,
                accessors,
            )
            .await
        else {
            if total_diagnostics.is_empty() && !adjacent_assertion_failed {
                return Ok(base_evaluated_locations);
            } else {
                return Err(crate::Invalid {
                    assertion_failed: adjacent_assertion_failed,
                    match_evidence: Default::default(),
                    diagnostics: total_diagnostics,
                    local_evaluated_locations: base_evaluated_locations,
                });
            }
        };

        total_diagnostics.extend(resolution_errors.into_iter().filter_map(|err| {
            crate::validate::schema_resolution_diagnostic(&err, value.range(), common_rules)
        }));

        let mut total_error = crate::Invalid::new();
        let mut matched = false;
        let mut matched_diagnostics = Vec::new();
        let mut matched_evaluated_locations = base_evaluated_locations;

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
                    matched = true;
                    matched_evaluated_locations.merge_from(result);
                }
                Err(error) => {
                    if error.assertion_failed {
                        total_error.combine(error);
                    } else {
                        matched = true;
                        matched_evaluated_locations
                            .merge_from(error.local_evaluated_locations.clone());
                        matched_diagnostics.extend(error.diagnostics);
                    }
                }
            }
        }

        if matched {
            if let Err(error) = validate_deprecated(
                any_of_schema.deprecation.as_ref(),
                accessors,
                value,
                Some(current_schema),
                schema_context,
                comment_directives,
                common_rules,
            ) {
                matched_diagnostics.extend(error.diagnostics);
            }

            matched_diagnostics.extend(total_diagnostics);

            if matched_diagnostics.is_empty() && !adjacent_assertion_failed {
                Ok(matched_evaluated_locations)
            } else {
                Err(crate::Invalid {
                    assertion_failed: adjacent_assertion_failed,
                    match_evidence: Default::default(),
                    diagnostics: matched_diagnostics,
                    local_evaluated_locations: matched_evaluated_locations,
                })
            }
        } else if !total_error.assertion_failed && total_diagnostics.is_empty() {
            Ok(matched_evaluated_locations)
        } else {
            total_error.prepend_diagnostics(total_diagnostics);
            total_error
                .local_evaluated_locations
                .merge_from(matched_evaluated_locations);
            Err(total_error)
        }
    }
    .boxed()
}
