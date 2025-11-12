use std::fmt::Debug;

use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, OneOfSchema, ValueSchema};
use tombi_severity_level::SeverityLevelDefaultError;

use super::Validate;
use crate::validate::{
    all_of::validate_all_of, any_of::validate_any_of, push_deprecated, type_mismatch,
};

pub fn validate_one_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    one_of_schema: &'a OneOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    common_rules: Option<&'a CommonLintRules>,
) -> BoxFuture<'b, Result<(), crate::Error>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    async move {
        let mut valid_count = 0;

        let mut schemas = one_of_schema.schemas.write().await;
        let mut each_results = Vec::with_capacity(schemas.len());
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

            let result = match (value.value_type(), current_schema.value_schema.as_ref()) {
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
                    value
                        .validate(accessors, Some(&current_schema), schema_context)
                        .await
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
                | (_, ValueSchema::Array(_)) => type_mismatch(
                    current_schema.value_schema.value_type().await,
                    value.value_type(),
                    value.range(),
                    common_rules,
                ),
                (_, ValueSchema::OneOf(one_of_schema)) => {
                    validate_one_of(
                        value,
                        accessors,
                        one_of_schema,
                        &current_schema,
                        schema_context,
                        common_rules,
                    )
                    .await
                }
                (_, ValueSchema::AnyOf(any_of_schema)) => {
                    validate_any_of(
                        value,
                        accessors,
                        any_of_schema,
                        &current_schema,
                        schema_context,
                        common_rules,
                    )
                    .await
                }
                (_, ValueSchema::AllOf(all_of_schema)) => {
                    validate_all_of(
                        value,
                        accessors,
                        all_of_schema,
                        &current_schema,
                        schema_context,
                        common_rules,
                    )
                    .await
                }
            };

            match &result {
                Ok(()) => {
                    valid_count += 1;
                }
                Err(error) => {
                    if error
                        .diagnostics
                        .iter()
                        .filter(|d| d.level() == tombi_diagnostic::Level::ERROR)
                        .count()
                        == 0
                    {
                        valid_count += 1;
                    }
                }
            }

            each_results.push(result);
        }

        if valid_count == 1 {
            for result in each_results {
                match result {
                    Ok(()) => return Ok(()),
                    Err(error) => {
                        if error
                            .diagnostics
                            .iter()
                            .filter(|d| d.level() == tombi_diagnostic::Level::ERROR)
                            .count()
                            == 0
                        {
                            return Err(error);
                        }
                    }
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

            if error.diagnostics.is_empty() && one_of_schema.deprecated == Some(true) {
                push_deprecated(&mut error.diagnostics, accessors, value, common_rules);
            }

            if error
                .diagnostics
                .iter()
                .filter(|d| d.level() == tombi_diagnostic::Level::ERROR)
                .count()
                == 0
                && valid_count > 1
            {
                let mut diagnostics = vec![];

                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::OneOfMultipleMatch {
                        valid_count,
                        total_count: schemas.len(),
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
