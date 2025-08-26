use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;

use crate::validate_ast::{
    all_of::validate_all_of, any_of::validate_any_of, one_of::validate_one_of, type_mismatch,
    Validate, ValueImpl,
};

impl Validate for tombi_ast::LocalDateTime {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let Some(token) = self.token() else {
                return Ok(());
            };

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::LocalDateTime(local_date_time_schema) => {
                        validate_local_date_time(
                            token.text(),
                            self.range(),
                            local_date_time_schema,
                            accessors,
                        )
                    }
                    tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
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
                    tombi_schema_store::ValueSchema::AnyOf(any_of_schema) => {
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
                    tombi_schema_store::ValueSchema::AllOf(all_of_schema) => {
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
                    tombi_schema_store::ValueSchema::Null => return Ok(()),
                    value_schema => {
                        type_mismatch(ValueType::Float, self.range(), value_schema).await
                    }
                }
            } else {
                Ok(())
            }
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::LocalDateTime {
    fn value_type(&self) -> ValueType {
        ValueType::LocalDateTime
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

fn validate_local_date_time<'a: 'b, 'b>(
    string_value: &'a str,
    range: tombi_text::Range,
    local_date_time_schema: &tombi_schema_store::LocalDateTimeSchema,
    accessors: &[tombi_schema_store::SchemaAccessor],
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];

    // Validate const value
    if let Some(const_value) = &local_date_time_schema.const_value {
        if string_value != *const_value {
            crate::Error {
                kind: crate::ErrorKind::Const {
                    expected: const_value.clone(),
                    actual: string_value.to_string(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(enumerate) = &local_date_time_schema.enumerate {
        if enumerate.iter().any(|s| s != string_value) {
            crate::Error {
                kind: crate::ErrorKind::Enumerate {
                    expected: enumerate.iter().map(ToString::to_string).collect(),
                    actual: string_value.to_string(),
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
                    tombi_schema_store::SchemaAccessors::new(accessors.to_vec()),
                    string_value.to_string(),
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
