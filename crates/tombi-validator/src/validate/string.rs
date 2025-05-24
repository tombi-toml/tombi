use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::ValueImpl;
use futures::{future::BoxFuture, FutureExt};
use regex::Regex;
use tombi_schema_store::ValueType;

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::String {
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
                    ValueType::String
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

                let string_schema = match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::String(string_schema) => string_schema,
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
                    _ => unreachable!("Expected a String schema"),
                };

                let value = self.value().to_string();
                if let Some(enumerate) = &string_schema.enumerate {
                    if !enumerate.contains(&value) {
                        crate::Error {
                            kind: crate::ErrorKind::Eunmerate {
                                expected: enumerate.iter().map(|s| format!("\"{s}\"")).collect(),
                                actual: value.clone(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(max_length) = &string_schema.max_length {
                    if value.len() > *max_length {
                        crate::Error {
                            kind: crate::ErrorKind::MaximumLength {
                                maximum: *max_length,
                                actual: value.len(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(min_length) = &string_schema.min_length {
                    if value.len() < *min_length {
                        crate::Error {
                            kind: crate::ErrorKind::MinimumLength {
                                minimum: *min_length,
                                actual: value.len(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(pattern) = &string_schema.pattern {
                    if let Ok(regex) = Regex::new(pattern) {
                        if !regex.is_match(&value) {
                            crate::Error {
                                kind: crate::ErrorKind::Pattern {
                                    pattern: pattern.clone(),
                                    actual: value,
                                },
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                        }
                    } else {
                        tracing::error!("Invalid regex pattern: {:?}", pattern);
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
