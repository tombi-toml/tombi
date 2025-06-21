use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::{OffsetDateTime, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for OffsetDateTime {
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
                    ValueType::OffsetDateTime
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

                let offset_date_time_schema = match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::OffsetDateTime(offset_date_time_schema) => {
                        offset_date_time_schema
                    }
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
                    _ => unreachable!("Expected an OffsetDateTime schema"),
                };

                let value_string = self.node().to_string();

                if let Some(const_value) = &offset_date_time_schema.const_value {
                    if value_string != *const_value {
                        crate::Error {
                            kind: crate::ErrorKind::Const {
                                expected: const_value.clone(),
                                actual: value_string.clone(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(enumerate) = &offset_date_time_schema.enumerate {
                    if !enumerate.contains(&value_string) {
                        crate::Error {
                            kind: crate::ErrorKind::Enumerate {
                                expected: enumerate.iter().map(ToString::to_string).collect(),
                                actual: value_string,
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
