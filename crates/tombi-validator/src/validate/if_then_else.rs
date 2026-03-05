use tombi_document_tree::ValueImpl;
use tombi_schema_store::CurrentSchema;

use crate::Validate;

pub async fn validate_if_then_else<T>(
    value: &T,
    accessors: &[tombi_schema_store::Accessor],
    if_then_else_schema: &tombi_schema_store::IfThenElseSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Result<(), crate::Error>
where
    T: Validate + ValueImpl + Sync + Send,
{
    // Resolve and validate the `if` schema
    let if_result = if let Ok(Some(if_current_schema)) = if_then_else_schema
        .if_schema
        .write()
        .await
        .resolve(
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
        return Ok(());
    };

    // Per JSON Schema spec: the `if` validation result itself is not exposed to the user.
    // If `if` succeeds, apply `then`; if `if` fails, apply `else`.
    if if_result.is_ok() {
        // `if` matched → apply `then` schema if present
        if let Some(then_schema) = &if_then_else_schema.then_schema {
            if let Ok(Some(then_current_schema)) = then_schema
                .write()
                .await
                .resolve(
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| log::warn!("{err}"))
            {
                return value
                    .validate(accessors, Some(&then_current_schema), schema_context)
                    .await;
            }
        }
    } else {
        // `if` did not match → apply `else` schema if present
        if let Some(else_schema) = &if_then_else_schema.else_schema {
            if let Ok(Some(else_current_schema)) = else_schema
                .write()
                .await
                .resolve(
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
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
    }

    Ok(())
}
