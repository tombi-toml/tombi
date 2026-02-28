use std::fmt::Debug;

use tombi_ast::TombiValueCommentDirective;
use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, OneOfSchema};
use tombi_severity_level::SeverityLevelDefaultError;

use super::Validate;
use crate::validate::{
    handle_deprecated, has_error_level_diagnostics, is_success_or_warning,
    not_schema::validate_not, validate_resolved_schema,
};

pub fn validate_one_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    one_of_schema: &'a OneOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_directives: Option<&'a [TombiValueCommentDirective]>,
    common_rules: Option<&'a CommonLintRules>,
) -> BoxFuture<'b, Result<(), crate::Error>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    async move {
        if let Some(not_schema) = one_of_schema.not.as_ref() {
            validate_not(
                value,
                accessors,
                not_schema,
                current_schema,
                schema_context,
                comment_directives,
                common_rules,
            )
            .await?;
        }

        let mut valid_count = 0;

        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            &one_of_schema.schemas,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
        )
        .await
        else {
            return Ok(());
        };
        let total_count = resolved_schemas.len();

        let mut each_results = Vec::with_capacity(resolved_schemas.len());
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

            if is_success_or_warning(&result) {
                valid_count += 1;
            }

            each_results.push(result);
        }

        if valid_count == 1 {
            for result in each_results {
                match result {
                    Ok(()) => return Ok(()),
                    Err(error) if !has_error_level_diagnostics(&error) => return Err(error),
                    Err(_) => {}
                }
            }

            unreachable!("one_of_schema must have exactly one valid schema");
        } else {
            let mut error = each_results
                .into_iter()
                .fold(crate::Error::new(), |mut a, b| {
                    if let Err(error) = b {
                        a.combine(error);
                    }
                    a
                });

            if error.diagnostics.is_empty() {
                handle_deprecated(
                    &mut error.diagnostics,
                    one_of_schema.deprecated,
                    accessors,
                    value,
                    comment_directives,
                    common_rules,
                );
            }

            if !has_error_level_diagnostics(&error) && valid_count > 1 {
                let mut diagnostics = vec![];

                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::OneOfMultipleMatch {
                        valid_count,
                        total_count,
                    }),
                    range: value.range(),
                }
                .push_diagnostic_with_level(
                    common_rules
                        .and_then(|rules| rules.one_of_multiple_match.as_ref())
                        .map(SeverityLevelDefaultError::from)
                        .unwrap_or_default(),
                    &mut diagnostics,
                );

                Err(diagnostics.into())
            } else {
                Err(error)
            }
        }
    }
    .boxed()
}
