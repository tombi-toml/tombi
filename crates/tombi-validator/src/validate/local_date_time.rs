use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::{LocalDateTime, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;

use crate::validate::type_mismatch;

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for LocalDateTime {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::LocalDateTime(local_date_time_schema) => {
                        validate_local_date_time(self, accessors, local_date_time_schema).await
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        validate_any_of(
                            self,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        validate_all_of(
                            self,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::Null => return Ok(()),
                    value_schema => type_mismatch(
                        value_schema.value_type().await,
                        self.value_type(),
                        self.range(),
                    ),
                }
            } else {
                Ok(())
            }
        }
        .boxed()
    }
}

async fn validate_local_date_time(
    local_date_time_value: &LocalDateTime,
    accessors: &[tombi_schema_store::Accessor],
    local_date_time_schema: &tombi_schema_store::LocalDateTimeSchema,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    let value_string = local_date_time_value.node().to_string();
    let range = local_date_time_value.range();

    if let Some(const_value) = &local_date_time_schema.const_value {
        if value_string != *const_value {
            crate::Error {
                kind: crate::ErrorKind::Const {
                    expected: const_value.clone(),
                    actual: value_string.clone(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(enumerate) = &local_date_time_schema.enumerate {
        if !enumerate.contains(&value_string) {
            crate::Error {
                kind: crate::ErrorKind::Enumerate {
                    expected: enumerate.iter().map(ToString::to_string).collect(),
                    actual: value_string.clone(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        if local_date_time_schema.deprecated == Some(true) {
            crate::Warning {
                kind: Box::new(crate::WarningKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::from(accessors),
                    value_string,
                )),
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
