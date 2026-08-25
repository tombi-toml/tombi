use std::fmt::Debug;

use tombi_ast_syntax::TombiValueCommentDirective;
use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree_syntax::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::CurrentSchema;

use crate::validate::{
    handle_deprecated, if_then_else::validate_if_then_else, not_schema::validate_not,
};

use super::Validate;

pub fn validate_all_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_directives: Option<&'a [TombiValueCommentDirective]>,
    common_rules: Option<&'a CommonLintRules>,
) -> BoxFuture<'b, Result<crate::Valid, crate::Invalid>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    log::trace!("value = {:?}", value);
    log::trace!("all_of_schema = {:?}", all_of_schema);

    async move {
        let mut total_diagnostics = vec![];
        let mut assertion_failed = false;
        let mut match_evidence = Box::<crate::MatchEvidence>::default();
        let mut evaluated_locations = crate::Valid::new();

        let Some((resolved_schemas, resolution_errors)) =
            tombi_schema_store::resolve_and_collect_schemas_with_errors(
                &all_of_schema.schemas,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                current_schema.strict,
                schema_context.store,
                &schema_context.schema_visits,
                accessors,
            )
            .await
        else {
            return Ok(crate::Valid::new());
        };

        total_diagnostics.extend(resolution_errors.into_iter().filter_map(|err| {
            crate::validate::schema_resolution_diagnostic(&err, value.range(), common_rules)
        }));

        for resolved_schema in &resolved_schemas {
            match value
                .validate(accessors, Some(resolved_schema), schema_context)
                .await
            {
                Ok(result) => evaluated_locations.merge_from(result),
                Err(error) => {
                    if !error.assertion_failed {
                        evaluated_locations.merge_from(error.local_evaluated_locations.clone());
                    }
                    assertion_failed |= error.assertion_failed;
                    total_diagnostics.extend(error.diagnostics);
                    match_evidence.merge_from(*error.match_evidence);
                }
            }
        }

        if total_diagnostics.is_empty() {
            handle_deprecated(
                &mut total_diagnostics,
                all_of_schema.deprecation.as_ref(),
                accessors,
                value,
                Some(current_schema),
                schema_context,
                comment_directives,
                common_rules,
            );
        }

        if let Some(not_schema) = all_of_schema.not.as_ref()
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
            assertion_failed |= error.assertion_failed;
            total_diagnostics.extend(error.diagnostics);
        }

        if let Some(if_then_else_schema) = all_of_schema.if_then_else.as_ref() {
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
                Ok(result) => evaluated_locations.merge_from(result),
                Err(error) => {
                    if !error.assertion_failed {
                        evaluated_locations.merge_from(error.local_evaluated_locations.clone());
                    }
                    assertion_failed |= error.assertion_failed;
                    total_diagnostics.extend(error.diagnostics);
                    match_evidence.merge_from(*error.match_evidence);
                }
            }
        }

        if total_diagnostics.is_empty() && !assertion_failed {
            Ok(evaluated_locations)
        } else {
            match_evidence.merge_from(*evaluated_locations.match_evidence.clone());
            Err(crate::Invalid {
                assertion_failed,
                match_evidence,
                diagnostics: total_diagnostics,
                local_evaluated_locations: evaluated_locations,
            })
        }
    }
    .boxed()
}
