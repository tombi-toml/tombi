use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::ValueImpl;
use futures::{future::BoxFuture, FutureExt};
use tombi_schema_store::ValueType;

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::Float {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = vec![];
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.value_type().await {
                    ValueType::Float
                    | ValueType::OneOf(_)
                    | ValueType::AnyOf(_)
                    | ValueType::AllOf(_) => {}
                    ValueType::Null => return Ok(()),
                    value_schema => {
                        crate::Error {
                            kind: crate::ErrorKind::TypeMismatch {
                                expected: value_schema,
                                actual: self.value_type(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);

                        return Err(diagnostics);
                    }
                }

                let float_schema = match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::Float(float_schema) => float_schema,
                    tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
                        return validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
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
                        )
                        .await
                    }
                    _ => unreachable!("Expected a Float schema"),
                };

                let value = self.value();
                if let Some(enumerate) = &float_schema.enumerate {
                    if !enumerate.contains(&value) {
                        crate::Error {
                            kind: crate::ErrorKind::Eunmerate {
                                expected: enumerate.iter().map(ToString::to_string).collect(),
                                actual: value.to_string(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(maximum) = &float_schema.maximum {
                    if value > *maximum {
                        crate::Error {
                            kind: crate::ErrorKind::MaximumFloat {
                                maximum: *maximum,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(minimum) = &float_schema.minimum {
                    if value < *minimum {
                        crate::Error {
                            kind: crate::ErrorKind::MinimumFloat {
                                minimum: *minimum,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(exclusive_maximum) = &float_schema.exclusive_maximum {
                    if value >= *exclusive_maximum {
                        crate::Error {
                            kind: crate::ErrorKind::ExclusiveMaximumFloat {
                                maximum: *exclusive_maximum,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(exclusive_minimum) = &float_schema.exclusive_minimum {
                    if value <= *exclusive_minimum {
                        crate::Error {
                            kind: crate::ErrorKind::ExclusiveMinimumFloat {
                                minimum: *exclusive_minimum,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(multiple_of) = &float_schema.multiple_of {
                    if (value % *multiple_of).abs() > f64::EPSILON {
                        crate::Error {
                            kind: crate::ErrorKind::MultipleOfFloat {
                                multiple_of: *multiple_of,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
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
