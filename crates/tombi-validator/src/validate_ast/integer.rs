use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::support::integer::{
    try_from_binary, try_from_decimal, try_from_hexadecimal, try_from_octal,
};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;

use crate::validate_ast::all_of::validate_all_of;
use crate::validate_ast::any_of::validate_any_of;
use crate::validate_ast::one_of::validate_one_of;
use crate::validate_ast::{validate_float_schema, ValueImpl};
use crate::Validate;

impl Validate for tombi_ast::IntegerBin {
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
            let Ok(value) = try_from_binary(token.text()) else {
                return Ok(());
            };

            validate_integer(
                value,
                self,
                accessors,
                current_schema,
                schema_context,
                comment_context,
            )
            .await
        }
        .boxed()
    }
}

impl Validate for tombi_ast::IntegerDec {
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
            let Ok(value) = try_from_decimal(token.text()) else {
                return Ok(());
            };

            validate_integer(
                value,
                self,
                accessors,
                current_schema,
                schema_context,
                comment_context,
            )
            .await
        }
        .boxed()
    }
}

impl Validate for tombi_ast::IntegerHex {
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
            let Ok(value) = try_from_hexadecimal(token.text()) else {
                return Ok(());
            };

            validate_integer(
                value,
                self,
                accessors,
                current_schema,
                schema_context,
                comment_context,
            )
            .await
        }
        .boxed()
    }
}

impl Validate for tombi_ast::IntegerOct {
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
            let Ok(value) = try_from_octal(token.text()) else {
                return Ok(());
            };

            validate_integer(
                value,
                self,
                accessors,
                current_schema,
                schema_context,
                comment_context,
            )
            .await
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::IntegerBin {
    fn value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl ValueImpl for tombi_ast::IntegerDec {
    fn value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl ValueImpl for tombi_ast::IntegerHex {
    fn value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl ValueImpl for tombi_ast::IntegerOct {
    fn value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

fn validate_integer<'a: 'b, 'b, T>(
    integer_value: i64,
    value: &'a T,
    accessors: &'a [tombi_schema_store::SchemaAccessor],
    current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
    schema_context: &'a tombi_schema_store::SchemaContext,
    comment_context: &'a CommentContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    async move {
        let mut diagnostics = vec![];

        if let Some(current_schema) = current_schema {
            match current_schema.value_schema.as_ref() {
                tombi_schema_store::ValueSchema::Integer(integer_schema) => {
                    validate_integer_schema(integer_value, value.range(), integer_schema, accessors)
                }
                tombi_schema_store::ValueSchema::Float(float_schema) => validate_float_schema(
                    integer_value as f64,
                    float_schema,
                    value.range(),
                    accessors,
                ),
                tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
                    validate_one_of(
                        value,
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
                        value,
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
                        value,
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
                            actual: ValueType::Integer,
                        },
                        range: value.range(),
                    }
                    .set_diagnostics(&mut diagnostics);
                    Err(diagnostics)
                }
            }
        } else {
            Ok(())
        }
    }
    .boxed()
}

// Helper function to validate integer against integer schema
fn validate_integer_schema(
    value: i64,
    range: tombi_text::Range,
    integer_schema: &tombi_schema_store::IntegerSchema,
    accessors: &[tombi_schema_store::SchemaAccessor],
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];

    // Validate const value
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

    // Validate enum
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

    // Validate maximum
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

    // Validate minimum
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

    // Validate exclusive maximum
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

    // Validate exclusive minimum
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

    // Validate multiple of
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

    // Check deprecated
    if diagnostics.is_empty() && integer_schema.deprecated == Some(true) {
        crate::Warning {
            kind: Box::new(crate::WarningKind::DeprecatedValue(
                tombi_schema_store::SchemaAccessors::new(accessors.to_vec()),
                value.to_string(),
            )),
            range,
        }
        .set_diagnostics(&mut diagnostics);
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
