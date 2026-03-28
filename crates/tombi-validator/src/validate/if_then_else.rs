use std::borrow::Cow;

use tombi_document_tree::ValueImpl;
use tombi_schema_store::CurrentSchema;

use crate::Validate;
use crate::validate::{
    has_error_level_diagnostics, is_assertion_success, merge_validation_results,
};

pub async fn validate_if_then_else<T>(
    value: &T,
    accessors: &[tombi_schema_store::Accessor],
    if_then_else_schema: &tombi_schema_store::IfThenElseSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Result<crate::EvaluatedLocations, crate::Error>
where
    T: Validate + ValueImpl + Sync + Send,
{
    let merge_if_result =
        |branch_result: Result<crate::EvaluatedLocations, crate::Error>,
         if_result: Result<crate::EvaluatedLocations, crate::Error>| match if_result {
            Ok(evaluated_locations) => {
                merge_validation_results(Ok(evaluated_locations), branch_result)
            }
            Err(error) if !has_error_level_diagnostics(&error) => {
                merge_validation_results(Err(error), branch_result)
            }
            Err(_) => branch_result,
        };

    // Resolve and validate the `if` schema
    let if_result = if let Ok(Some(if_current_schema)) = tombi_schema_store::resolve_schema_item(
        &if_then_else_schema.if_schema,
        current_schema.schema_uri.clone(),
        current_schema.definitions.clone(),
        schema_context.store,
    )
    .await
    .inspect_err(|err| log::warn!("{err}"))
    {
        value
            .validate(accessors, Some(&if_current_schema), schema_context)
            .await
    } else {
        return Ok(crate::EvaluatedLocations::new());
    };

    // Per JSON Schema spec: branching is based on assertion result.
    if is_assertion_success(&if_result) {
        // `if` matched → apply `then` schema if present
        if let Some(then_schema) = &if_then_else_schema.then_schema
            && let Ok(Some(then_current_schema)) = tombi_schema_store::resolve_schema_item(
                then_schema,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
            .inspect_err(|err| log::warn!("{err}"))
        {
            let branch_result = value
                .validate(accessors, Some(&then_current_schema), schema_context)
                .await;
            return merge_if_result(branch_result, if_result);
        }

        return merge_if_result(Ok(crate::EvaluatedLocations::new()), if_result);
    } else {
        // `if` did not match → apply `else` schema if present
        if let Some(else_schema) = &if_then_else_schema.else_schema
            && let Ok(Some(else_current_schema)) = tombi_schema_store::resolve_schema_item(
                else_schema,
                Cow::Borrowed(current_schema.schema_uri.as_ref()),
                Cow::Borrowed(current_schema.definitions.as_ref()),
                schema_context.store,
            )
            .await
            .inspect_err(|err| log::warn!("{err}"))
        {
            return value
                .validate(accessors, Some(&else_current_schema), schema_context)
                .await;
        }
    }

    Ok(crate::EvaluatedLocations::new())
}
