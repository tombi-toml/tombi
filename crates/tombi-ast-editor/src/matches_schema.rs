use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{AllOfSchema, AnyOfSchema, CurrentSchema, OneOfSchema, ValueSchema};

/// Lightweight schema matching for the AST editor.
///
/// Unlike full validation ([`tombi_validator::Validate`]), this only checks
/// structural compatibility — type matching for primitives, and required
/// key presence for tables. This avoids the exponential complexity that
/// occurs when full recursive validation is used just to determine which
/// schema variant from oneOf/allOf/anyOf applies to a given value.
pub(crate) fn matches_schema_value<'a: 'b, 'b>(
    value: &'a tombi_document_tree::Value,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> BoxFuture<'b, bool> {
    async move {
        match (value, current_schema.value_schema.as_ref()) {
            // Primitive type matching — just verify types are compatible
            (tombi_document_tree::Value::Boolean(_), ValueSchema::Boolean(_))
            | (
                tombi_document_tree::Value::Integer(_),
                ValueSchema::Integer(_) | ValueSchema::Float(_),
            )
            | (tombi_document_tree::Value::Float(_), ValueSchema::Float(_))
            | (tombi_document_tree::Value::String(_), ValueSchema::String(_))
            | (
                tombi_document_tree::Value::OffsetDateTime(_),
                ValueSchema::OffsetDateTime(_),
            )
            | (
                tombi_document_tree::Value::LocalDateTime(_),
                ValueSchema::LocalDateTime(_),
            )
            | (tombi_document_tree::Value::LocalDate(_), ValueSchema::LocalDate(_))
            | (tombi_document_tree::Value::LocalTime(_), ValueSchema::LocalTime(_))
            | (tombi_document_tree::Value::Array(_), ValueSchema::Array(_)) => true,

            // For tables, check required keys are present (shallow — no child validation)
            (tombi_document_tree::Value::Table(table), ValueSchema::Table(table_schema)) => {
                if let Some(required) = &table_schema.required {
                    required.iter().all(|key| table.contains_key(key.as_str()))
                } else {
                    true
                }
            }

            // Null schema matches nothing
            (_, ValueSchema::Null) => false,

            // Composition schemas — recurse with shallow matching
            (_, ValueSchema::OneOf(one_of_schema)) => {
                matches_one_of_value(value, one_of_schema, current_schema, schema_context).await
            }
            (_, ValueSchema::AnyOf(any_of_schema)) => {
                matches_any_of_value(value, any_of_schema, current_schema, schema_context).await
            }
            (_, ValueSchema::AllOf(all_of_schema)) => {
                matches_all_of_value(value, all_of_schema, current_schema, schema_context).await
            }

            // Type mismatch
            _ => false,
        }
    }
    .boxed()
}

/// Lightweight schema matching for arrays.
///
/// Since arrays always type-match against array schemas, the main purpose
/// of this function is to handle composition schemas (oneOf/allOf/anyOf)
/// that wrap the actual array schema.
pub(crate) fn matches_schema_array<'a: 'b, 'b>(
    array: &'a tombi_document_tree::Array,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> BoxFuture<'b, bool> {
    async move {
        match current_schema.value_schema.as_ref() {
            ValueSchema::Array(_) => true,

            ValueSchema::OneOf(one_of_schema) => {
                let schemas = one_of_schema.schemas.read().await.clone();
                for referable_schema in schemas.iter() {
                    let mut referable_schema = referable_schema.clone();
                    if let Ok(Some(resolved)) = referable_schema
                        .resolve(
                            current_schema.schema_uri.clone(),
                            current_schema.definitions.clone(),
                            schema_context.store,
                        )
                        .await
                    {
                        let resolved = resolved.into_owned();
                        if matches_schema_array(array, &resolved, schema_context).await {
                            return true;
                        }
                    }
                }
                false
            }

            ValueSchema::AnyOf(any_of_schema) => {
                let schemas = any_of_schema.schemas.read().await.clone();
                for referable_schema in schemas.iter() {
                    let mut referable_schema = referable_schema.clone();
                    if let Ok(Some(resolved)) = referable_schema
                        .resolve(
                            current_schema.schema_uri.clone(),
                            current_schema.definitions.clone(),
                            schema_context.store,
                        )
                        .await
                    {
                        let resolved = resolved.into_owned();
                        if matches_schema_array(array, &resolved, schema_context).await {
                            return true;
                        }
                    }
                }
                false
            }

            ValueSchema::AllOf(all_of_schema) => {
                let schemas = all_of_schema.schemas.read().await.clone();
                for referable_schema in schemas.iter() {
                    let mut referable_schema = referable_schema.clone();
                    if let Ok(Some(resolved)) = referable_schema
                        .resolve(
                            current_schema.schema_uri.clone(),
                            current_schema.definitions.clone(),
                            schema_context.store,
                        )
                        .await
                    {
                        let resolved = resolved.into_owned();
                        if !matches_schema_array(array, &resolved, schema_context).await {
                            return false;
                        }
                    }
                }
                true
            }

            _ => false,
        }
    }
    .boxed()
}

/// oneOf: returns true if at least one variant matches (short-circuit)
async fn matches_one_of_value<'a>(
    value: &'a tombi_document_tree::Value,
    one_of_schema: &'a OneOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> bool {
    let schemas = one_of_schema.schemas.read().await.clone();
    for referable_schema in schemas.iter() {
        let mut referable_schema = referable_schema.clone();
        if let Ok(Some(resolved)) = referable_schema
            .resolve(
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
        {
            let resolved = resolved.into_owned();
            if matches_schema_value(value, &resolved, schema_context).await {
                return true;
            }
        }
    }
    false
}

/// anyOf: returns true if at least one variant matches (short-circuit)
async fn matches_any_of_value<'a>(
    value: &'a tombi_document_tree::Value,
    any_of_schema: &'a AnyOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> bool {
    let schemas = any_of_schema.schemas.read().await.clone();
    for referable_schema in schemas.iter() {
        let mut referable_schema = referable_schema.clone();
        if let Ok(Some(resolved)) = referable_schema
            .resolve(
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
        {
            let resolved = resolved.into_owned();
            if matches_schema_value(value, &resolved, schema_context).await {
                return true;
            }
        }
    }
    false
}

/// allOf: returns true only if ALL sub-schemas match (short-circuit on failure)
async fn matches_all_of_value<'a>(
    value: &'a tombi_document_tree::Value,
    all_of_schema: &'a AllOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> bool {
    let schemas = all_of_schema.schemas.read().await.clone();
    for referable_schema in schemas.iter() {
        let mut referable_schema = referable_schema.clone();
        if let Ok(Some(resolved)) = referable_schema
            .resolve(
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
        {
            let resolved = resolved.into_owned();
            if !matches_schema_value(value, &resolved, schema_context).await {
                return false;
            }
        }
    }
    true
}
