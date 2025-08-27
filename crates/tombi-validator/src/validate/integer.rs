use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;

use crate::validate::type_mismatch;

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::Integer {
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
                    tombi_schema_store::ValueSchema::Integer(integer_schema) => {
                        validate_integer_schema(self, accessors, integer_schema).await
                    }
                    tombi_schema_store::ValueSchema::Float(float_schema) => {
                        validate_float_schema_for_integer(self, accessors, float_schema).await
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

async fn validate_integer_schema(
    integer_value: &tombi_document_tree::Integer,
    accessors: &[tombi_schema_store::Accessor],
    integer_schema: &tombi_schema_store::IntegerSchema,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    let value = integer_value.value();
    let range = integer_value.range();

    if let Some(const_value) = &integer_schema.const_value {
        if value != *const_value {
            crate::Error {
                kind: crate::ErrorKind::Const {
                    expected: const_value.to_string(),
                    actual: value.to_string(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(enumerate) = &integer_schema.enumerate {
        if !enumerate.contains(&value) {
            crate::Error {
                kind: crate::ErrorKind::Enumerate {
                    expected: enumerate.iter().map(ToString::to_string).collect(),
                    actual: value.to_string(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(maximum) = &integer_schema.maximum {
        if value > *maximum {
            crate::Error {
                kind: crate::ErrorKind::IntegerMaximum {
                    maximum: *maximum,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(minimum) = &integer_schema.minimum {
        if value < *minimum {
            crate::Error {
                kind: crate::ErrorKind::IntegerMinimum {
                    minimum: *minimum,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(exclusive_maximum) = &integer_schema.exclusive_maximum {
        if value >= *exclusive_maximum {
            crate::Error {
                kind: crate::ErrorKind::IntegerExclusiveMaximum {
                    maximum: *exclusive_maximum - 1,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(exclusive_minimum) = &integer_schema.exclusive_minimum {
        if value <= *exclusive_minimum {
            crate::Error {
                kind: crate::ErrorKind::IntegerExclusiveMinimum {
                    minimum: *exclusive_minimum + 1,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(multiple_of) = &integer_schema.multiple_of {
        if value % *multiple_of != 0 {
            crate::Error {
                kind: crate::ErrorKind::IntegerMultipleOf {
                    multiple_of: *multiple_of,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        if integer_schema.deprecated == Some(true) {
            crate::Warning {
                kind: Box::new(crate::WarningKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::from(accessors),
                    value.to_string(),
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

async fn validate_float_schema_for_integer(
    integer_value: &tombi_document_tree::Integer,
    accessors: &[tombi_schema_store::Accessor],
    float_schema: &tombi_schema_store::FloatSchema,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    let value = integer_value.value() as f64;
    let range = integer_value.range();

    if let Some(const_value) = &float_schema.const_value {
        if (value - *const_value).abs() > f64::EPSILON {
            crate::Error {
                kind: crate::ErrorKind::Const {
                    expected: const_value.to_string(),
                    actual: value.to_string(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(enumerate) = &float_schema.enumerate {
        if !enumerate.contains(&value) {
            crate::Error {
                kind: crate::ErrorKind::Enumerate {
                    expected: enumerate.iter().map(ToString::to_string).collect(),
                    actual: value.to_string(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(maximum) = &float_schema.maximum {
        if value > *maximum {
            crate::Error {
                kind: crate::ErrorKind::FloatMaximum {
                    maximum: *maximum,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(minimum) = &float_schema.minimum {
        if value < *minimum {
            crate::Error {
                kind: crate::ErrorKind::FloatMinimum {
                    minimum: *minimum,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(exclusive_maximum) = &float_schema.exclusive_maximum {
        if value >= *exclusive_maximum {
            crate::Error {
                kind: crate::ErrorKind::FloatExclusiveMaximum {
                    maximum: *exclusive_maximum - 1.0,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(exclusive_minimum) = &float_schema.exclusive_minimum {
        if value <= *exclusive_minimum {
            crate::Error {
                kind: crate::ErrorKind::FloatExclusiveMinimum {
                    minimum: *exclusive_minimum + 1.0,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(multiple_of) = &float_schema.multiple_of {
        if value % *multiple_of != 0.0 {
            crate::Error {
                kind: crate::ErrorKind::FloatMultipleOf {
                    multiple_of: *multiple_of,
                    actual: value,
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        if float_schema.deprecated == Some(true) {
            crate::Warning {
                kind: Box::new(crate::WarningKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::from(accessors),
                    value.to_string(),
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
