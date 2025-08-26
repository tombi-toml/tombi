use itertools::Itertools;
use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::support::float::try_from_float;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;

use crate::validate_ast::{
    all_of::validate_all_of, any_of::validate_any_of, one_of::validate_one_of, type_mismatch,
    Validate, ValueImpl,
};

impl Validate for tombi_ast::Float {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let Some(token) = self.token() else {
                return Ok(());
            };

            let Ok(value) = try_from_float(token.text()) else {
                return Ok(());
            };

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::Float(float_schema) => {
                        validate_float_schema(value, float_schema, self.range(), accessors)
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

impl ValueImpl for tombi_ast::Float {
    fn value_type(&self) -> ValueType {
        ValueType::Float
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

// Helper function to validate float against float schema
pub(crate) fn validate_float_schema(
    value: f64,
    float_schema: &tombi_schema_store::FloatSchema,
    range: tombi_text::Range,
    accessors: &[tombi_schema_store::Accessor],
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    // Validate const value
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

    // Validate enum
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

    // Validate maximum
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

    // Validate minimum
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

    // Check deprecated
    if diagnostics.is_empty() && float_schema.deprecated == Some(true) {
        crate::Warning {
            kind: Box::new(crate::WarningKind::DeprecatedValue(
                tombi_schema_store::SchemaAccessors::new(
                    accessors.into_iter().map(Into::into).collect_vec(),
                ),
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
