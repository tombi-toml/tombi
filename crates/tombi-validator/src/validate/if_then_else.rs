use std::borrow::Cow;

use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree_syntax::ValueImpl;
use tombi_schema_store::CurrentSchema;

use crate::Validate;
use crate::validate::{discard_failed_annotations, is_assertion_success, merge_validation_results};

pub async fn validate_if_then_else<T>(
    value: &T,
    accessors: &[tombi_schema_store::Accessor],
    if_then_else_schema: &tombi_schema_store::IfThenElseSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    common_rules: Option<&CommonLintRules>,
) -> Result<crate::Valid, crate::Invalid>
where
    T: Validate + ValueImpl + Sync + Send,
{
    // A failed `if`, `then` or `else` subschema drops its own annotations, but
    // the ones its successful siblings produced survive, so the rule is applied
    // per subschema result rather than to the merged result.
    #[allow(clippy::result_large_err)]
    let merge_if_result =
        |mut branch_result: Result<crate::Valid, crate::Invalid>,
         if_result: Result<crate::Valid, crate::Invalid>| {
            discard_failed_annotations(&mut branch_result);
            match if_result {
                Ok(evaluated_locations) => {
                    merge_validation_results(Ok(evaluated_locations), branch_result)
                }
                Err(error) if !error.assertion_failed => {
                    merge_validation_results(Err(error), branch_result)
                }
                Err(_) => branch_result,
            }
        };

    // Resolve and validate the `if` schema
    let if_result = match tombi_schema_store::resolve_schema_item(
        &if_then_else_schema.if_schema,
        current_schema.schema_uri.clone(),
        current_schema.definitions.clone(),
        current_schema.strict,
        schema_context.store,
    )
    .await
    {
        Ok(Some(if_current_schema)) => {
            value
                .validate(accessors, Some(&if_current_schema), schema_context)
                .await
        }
        Err(err) => {
            if let Some(diagnostic) =
                crate::validate::schema_resolution_diagnostic(&err, value.range(), common_rules)
            {
                return Err(vec![diagnostic].into());
            }

            return Ok(crate::Valid::new());
        }
        Ok(None) => return Ok(crate::Valid::new()),
    };

    // Per JSON Schema spec: branching is based on assertion result.
    if is_assertion_success(&if_result) {
        // `if` matched → apply `then` schema if present
        if let Some(then_schema) = &if_then_else_schema.then_schema {
            match tombi_schema_store::resolve_schema_item(
                then_schema,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                current_schema.strict,
                schema_context.store,
            )
            .await
            {
                Ok(Some(then_current_schema)) => {
                    let branch_result = value
                        .validate(accessors, Some(&then_current_schema), schema_context)
                        .await;
                    return merge_if_result(branch_result, if_result);
                }
                Err(err) => {
                    if let Some(diagnostic) = crate::validate::schema_resolution_diagnostic(
                        &err,
                        value.range(),
                        common_rules,
                    ) {
                        return merge_if_result(Err(vec![diagnostic].into()), if_result);
                    }
                }
                Ok(None) => {}
            }
        }

        return merge_if_result(Ok(crate::Valid::new()), if_result);
    } else {
        // `if` did not match → apply `else` schema if present
        if let Some(else_schema) = &if_then_else_schema.else_schema {
            match tombi_schema_store::resolve_schema_item(
                else_schema,
                Cow::Borrowed(current_schema.schema_uri.as_ref()),
                Cow::Borrowed(current_schema.definitions.as_ref()),
                current_schema.strict,
                schema_context.store,
            )
            .await
            {
                Ok(Some(else_current_schema)) => {
                    let branch_result = value
                        .validate(accessors, Some(&else_current_schema), schema_context)
                        .await;
                    return merge_if_result(branch_result, if_result);
                }
                Err(err) => {
                    if let Some(diagnostic) = crate::validate::schema_resolution_diagnostic(
                        &err,
                        value.range(),
                        common_rules,
                    ) {
                        return merge_if_result(Err(vec![diagnostic].into()), if_result);
                    }
                }
                Ok(None) => {}
            }
        }
    }

    Ok(crate::Valid::new())
}
