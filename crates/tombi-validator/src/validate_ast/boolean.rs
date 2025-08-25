use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;

use crate::{
    validate_ast::{
        all_of::validate_all_of, any_of::validate_any_of, one_of::validate_one_of, ValueImpl,
    },
    Validate,
};

impl Validate for tombi_ast::Boolean {
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

            let value = match token.text() {
                "true" => true,
                "false" => false,
                _ => unreachable!("Invalid boolean value"),
            };

            let Some(range) = self.get_range() else {
                return Ok(());
            };

            let mut diagnostics = vec![];

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::Boolean(boolean_schema) => {
                        validate_boolean_schema(
                            value,
                            boolean_schema,
                            range,
                            accessors,
                            &mut diagnostics,
                        );
                    }
                    tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
                        return validate_one_of(
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
                        return validate_any_of(
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
                        return validate_all_of(
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
                    schema => {
                        crate::Error {
                            kind: crate::ErrorKind::TypeMismatch2 {
                                expected: schema.value_type().await,
                                actual: ValueType::Boolean,
                            },
                            range,
                        }
                        .set_diagnostics(&mut diagnostics);

                        return Err(diagnostics);
                    }
                }
            }

            if diagnostics.is_empty() {
                Ok(())
            } else {
                Err(diagnostics)
            }
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::Boolean {
    fn value_type(&self) -> ValueType {
        ValueType::Boolean
    }

    fn get_range(&self) -> Option<tombi_text::Range> {
        self.token().map(|token| token.range())
    }
}

fn validate_boolean_schema(
    value: bool,
    boolean_schema: &tombi_schema_store::BooleanSchema,
    range: tombi_text::Range,
    accessors: &[tombi_schema_store::SchemaAccessor],
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if let Some(const_value) = &boolean_schema.const_value {
        if value != *const_value {
            crate::Error {
                kind: crate::ErrorKind::Const {
                    expected: const_value.to_string(),
                    actual: value.to_string(),
                },
                range,
            }
            .set_diagnostics(diagnostics);
        }
    }

    if let Some(enumerate) = &boolean_schema.enumerate {
        if !enumerate.contains(&value) {
            crate::Error {
                kind: crate::ErrorKind::Enumerate {
                    expected: enumerate.iter().map(ToString::to_string).collect(),
                    actual: value.to_string(),
                },
                range,
            }
            .set_diagnostics(diagnostics);
        }
    }

    if diagnostics.is_empty() {
        if boolean_schema.deprecated == Some(true) {
            crate::Warning {
                kind: Box::new(crate::WarningKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::new(accessors.to_vec()),
                    value.to_string(),
                )),
                range,
            }
            .set_diagnostics(diagnostics);
        }
    }
}
