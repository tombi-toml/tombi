use std::fmt::Debug;

use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, OneOfSchema, ValueSchema};

use super::Validate;
use crate::validate::{all_of::validate_all_of, any_of::validate_any_of};

pub fn validate_one_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::SchemaAccessor],
    one_of_schema: &'a OneOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    async move {
        let mut valid_count = 0;

        let mut schemas = one_of_schema.schemas.write().await;
        let mut each_diagnostics = Vec::with_capacity(schemas.len());
        for referable_schema in schemas.iter_mut() {
            let Ok(Some(current_schema)) = referable_schema
                .resolve(
                    current_schema.schema_url.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
            else {
                continue;
            };

            let diagnostics = match (value.value_type(), current_schema.value_schema.as_ref()) {
                (tombi_document_tree::ValueType::Boolean, ValueSchema::Boolean(_))
                | (
                    tombi_document_tree::ValueType::Integer,
                    ValueSchema::Integer(_) | ValueSchema::Float(_),
                )
                | (tombi_document_tree::ValueType::Float, ValueSchema::Float(_))
                | (tombi_document_tree::ValueType::String, ValueSchema::String(_))
                | (
                    tombi_document_tree::ValueType::OffsetDateTime,
                    ValueSchema::OffsetDateTime(_),
                )
                | (tombi_document_tree::ValueType::LocalDateTime, ValueSchema::LocalDateTime(_))
                | (tombi_document_tree::ValueType::LocalDate, ValueSchema::LocalDate(_))
                | (tombi_document_tree::ValueType::LocalTime, ValueSchema::LocalTime(_))
                | (tombi_document_tree::ValueType::Table, ValueSchema::Table(_))
                | (tombi_document_tree::ValueType::Array, ValueSchema::Array(_)) => {
                    match value
                        .validate(accessors, Some(&current_schema), schema_context)
                        .await
                    {
                        Ok(()) => Vec::with_capacity(0),
                        Err(diagnostics) => diagnostics,
                    }
                }
                (_, ValueSchema::Null) => {
                    continue;
                }
                (_, ValueSchema::Boolean(_))
                | (_, ValueSchema::Integer(_))
                | (_, ValueSchema::Float(_))
                | (_, ValueSchema::String(_))
                | (_, ValueSchema::OffsetDateTime(_))
                | (_, ValueSchema::LocalDateTime(_))
                | (_, ValueSchema::LocalDate(_))
                | (_, ValueSchema::LocalTime(_))
                | (_, ValueSchema::Table(_))
                | (_, ValueSchema::Array(_)) => {
                    vec![crate::Error {
                        kind: crate::ErrorKind::TypeMismatch {
                            expected: current_schema.value_schema.value_type().await,
                            actual: value.value_type(),
                        },
                        range: value.range(),
                    }
                    .into()]
                }
                (_, ValueSchema::OneOf(one_of_schema)) => {
                    match validate_one_of(
                        value,
                        accessors,
                        one_of_schema,
                        &current_schema,
                        schema_context,
                    )
                    .await
                    {
                        Ok(()) => Vec::with_capacity(0),
                        Err(diagnostics) => diagnostics,
                    }
                }
                (_, ValueSchema::AnyOf(any_of_schema)) => {
                    match validate_any_of(
                        value,
                        accessors,
                        any_of_schema,
                        &current_schema,
                        schema_context,
                    )
                    .await
                    {
                        Ok(()) => Vec::with_capacity(0),
                        Err(diagnostics) => diagnostics,
                    }
                }
                (_, ValueSchema::AllOf(all_of_schema)) => {
                    match validate_all_of(
                        value,
                        accessors,
                        all_of_schema,
                        &current_schema,
                        schema_context,
                    )
                    .await
                    {
                        Ok(()) => Vec::with_capacity(0),
                        Err(diagnostics) => diagnostics,
                    }
                }
            };

            if diagnostics
                .iter()
                .filter(|d| d.level() == tombi_diagnostic::Level::ERROR)
                .count()
                == 0
            {
                valid_count += 1;
            }

            each_diagnostics.push(diagnostics);
        }

        if valid_count == 1 {
            for diagnostics in each_diagnostics {
                if diagnostics.is_empty() {
                    return Ok(());
                } else if diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.level() == tombi_diagnostic::Level::ERROR)
                    .count()
                    == 0
                {
                    return Err(diagnostics);
                }
            }

            unreachable!("one_of_schema must have exactly one valid schema");
        } else {
            Err(each_diagnostics.into_iter().flatten().collect())
        }
    }
    .boxed()
}
