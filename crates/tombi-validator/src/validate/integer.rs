use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::ValueImpl;
use futures::{future::BoxFuture, FutureExt};
use tombi_schema_store::ValueType;

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::Integer {
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
                    ValueType::Integer
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

                let integer_schema = match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::Integer(integer_schema) => integer_schema,
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
                    _ => unreachable!("Expected an Integer schema"),
                };

                let value = self.value();
                if let Some(enumerate) = &integer_schema.enumerate {
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

                if let Some(maximum) = &integer_schema.maximum {
                    if value > *maximum {
                        crate::Error {
                            kind: crate::ErrorKind::MaximumInteger {
                                maximum: *maximum,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(minimum) = &integer_schema.minimum {
                    if value < *minimum {
                        crate::Error {
                            kind: crate::ErrorKind::MinimumInteger {
                                minimum: *minimum,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(exclusive_maximum) = &integer_schema.exclusive_maximum {
                    if value >= *exclusive_maximum {
                        crate::Error {
                            kind: crate::ErrorKind::ExclusiveMaximumInteger {
                                maximum: *exclusive_maximum - 1,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(exclusive_minimum) = &integer_schema.exclusive_minimum {
                    if value <= *exclusive_minimum {
                        crate::Error {
                            kind: crate::ErrorKind::ExclusiveMinimumInteger {
                                minimum: *exclusive_minimum + 1,
                                actual: value,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(multiple_of) = &integer_schema.multiple_of {
                    if value % *multiple_of != 0 {
                        crate::Error {
                            kind: crate::ErrorKind::MultipleOfInteger {
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
