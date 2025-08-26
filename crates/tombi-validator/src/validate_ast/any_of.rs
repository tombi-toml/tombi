use std::fmt::Debug;

use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, ValueSchema, ValueType};

use crate::validate_ast::all_of::validate_all_of;
use crate::validate_ast::one_of::validate_one_of;
use crate::validate_ast::{Validate, ValueImpl};

pub fn validate_any_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    any_of_schema: &'a tombi_schema_store::AnyOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_context: &'a CommentContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    tracing::trace!("value = {:?}", value);
    tracing::trace!("any_of_schema = {:?}", any_of_schema);

    async move {
        let mut schemas = any_of_schema.schemas.write().await;
        let mut total_diagnostics = Vec::new();

        for referable_schema in schemas.iter_mut() {
            let current_schema = if let Ok(Some(current_schema)) = referable_schema
                .resolve(
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| tracing::warn!("{err}"))
            {
                current_schema
            } else {
                continue;
            };

            let diagnostics = match (value.value_type(), current_schema.value_schema.as_ref()) {
                (ValueType::Boolean, ValueSchema::Boolean(_))
                | (ValueType::Integer, ValueSchema::Integer(_) | ValueSchema::Float(_))
                | (ValueType::Float, ValueSchema::Float(_))
                | (ValueType::String, ValueSchema::String(_))
                | (ValueType::OffsetDateTime, ValueSchema::OffsetDateTime(_))
                | (ValueType::LocalDateTime, ValueSchema::LocalDateTime(_))
                | (ValueType::LocalDate, ValueSchema::LocalDate(_))
                | (ValueType::LocalTime, ValueSchema::LocalTime(_))
                | (ValueType::Table, ValueSchema::Table(_))
                | (ValueType::Array, ValueSchema::Array(_)) => {
                    match value
                        .validate(
                            accessors,
                            Some(&current_schema),
                            schema_context,
                            comment_context,
                        )
                        .await
                    {
                        Ok(()) => {
                            return Ok(());
                        }
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
                        kind: crate::ErrorKind::TypeMismatch2 {
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
                        comment_context,
                    )
                    .await
                    {
                        Ok(()) => {
                            return Ok(());
                        }
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
                        comment_context,
                    )
                    .await
                    {
                        Ok(()) => {
                            return Ok(());
                        }
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
                        comment_context,
                    )
                    .await
                    {
                        Ok(()) => {
                            return Ok(());
                        }
                        Err(diagnostics) => diagnostics,
                    }
                }
            };

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

            total_diagnostics.extend(diagnostics);
        }

        Err(total_diagnostics)
    }
    .boxed()
}
