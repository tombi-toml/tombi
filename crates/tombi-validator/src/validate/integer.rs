use tombi_comment_directive::value::IntegerCommonRules;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;
use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};

use crate::{
    comment_directive::get_tombi_key_table_value_rules_and_diagnostics,
    validate::type_mismatch,
};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::Integer {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut total_diagnostics = vec![];
            let value_rules = if let Some(comment_directives) = self.comment_directives() {
                let (value_rules, diagnostics) =
                    get_tombi_key_table_value_rules_and_diagnostics::<IntegerCommonRules>(
                        comment_directives,
                        accessors,
                    )
                    .await;

                total_diagnostics.extend(diagnostics);

                value_rules
            } else {
                None
            };

            if let Some(current_schema) = current_schema {
                let result = match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::Integer(integer_schema) => {
                        validate_integer_schema(
                            self,
                            accessors,
                            integer_schema,
                            value_rules.as_ref(),
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::Float(float_schema) => {
                        validate_float_schema_for_integer(
                            self,
                            accessors,
                            float_schema,
                            value_rules.as_ref(),
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
                        return validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            value_rules.as_ref().map(|rules| &rules.common),
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
                            value_rules.as_ref().map(|rules| &rules.common),
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
                            value_rules.as_ref().map(|rules| &rules.common),
                        )
                        .await
                    }
                    ValueSchema::Null => return Ok(()),
                    value_schema => type_mismatch(
                        value_schema.value_type().await,
                        self.value_type(),
                        self.range(),
                        value_rules.as_ref().map(|rules| &rules.common),
                    ),
                };

                if let Err(diagnostics) = result {
                    total_diagnostics.extend(diagnostics);
                }
            }

            if total_diagnostics.is_empty() {
                Ok(())
            } else {
                Err(total_diagnostics)
            }
        }
        .boxed()
    }
}

async fn validate_integer_schema(
    integer_value: &tombi_document_tree::Integer,
    accessors: &[tombi_schema_store::Accessor],
    integer_schema: &tombi_schema_store::IntegerSchema,
    value_rules: Option<&IntegerCommonRules>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    let value = integer_value.value();
    let range = integer_value.range();

    if let Some(const_value) = &integer_schema.const_value {
        let level = value_rules
            .map(|rules| &rules.common)
            .and_then(|rules| {
                rules
                    .const_value
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        if value != *const_value {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::Const {
                    expected: const_value.to_string(),
                    actual: value.to_string(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(enumerate) = &integer_schema.enumerate {
        let level = value_rules
            .map(|rules| &rules.common)
            .and_then(|rules| {
                rules
                    .enumerate
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        if !enumerate.contains(&value) {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::Enumerate {
                    expected: enumerate.iter().map(ToString::to_string).collect(),
                    actual: value.to_string(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(maximum) = &integer_schema.maximum {
        let level = value_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .integer_maximum
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        if value > *maximum {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerMaximum {
                    maximum: *maximum,
                    actual: value,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(minimum) = &integer_schema.minimum {
        let level = value_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .integer_minimum
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        if value < *minimum {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerMinimum {
                    minimum: *minimum,
                    actual: value,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(exclusive_maximum) = &integer_schema.exclusive_maximum {
        let level = value_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .integer_exclusive_maximum
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        if value >= *exclusive_maximum {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerExclusiveMaximum {
                    maximum: *exclusive_maximum - 1,
                    actual: value,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(exclusive_minimum) = &integer_schema.exclusive_minimum {
        let level = value_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .integer_exclusive_minimum
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        if value <= *exclusive_minimum {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerExclusiveMinimum {
                    minimum: *exclusive_minimum + 1,
                    actual: value,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(multiple_of) = &integer_schema.multiple_of {
        let level = value_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .integer_multiple_of
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        if value % *multiple_of != 0 {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerMultipleOf {
                    multiple_of: *multiple_of,
                    actual: value,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        if integer_schema.deprecated == Some(true) {
            let level = value_rules
                .map(|rules| &rules.common)
                .and_then(|rules| {
                    rules
                        .deprecated
                        .as_ref()
                        .map(SeverityLevelDefaultWarn::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::from(accessors),
                    value.to_string(),
                )),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
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
    value_rules: Option<&IntegerCommonRules>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    let value = integer_value.value() as f64;
    let range = integer_value.range();

    if let Some(const_value) = &float_schema.const_value {
        if (value - *const_value).abs() > f64::EPSILON {
            let level = value_rules
                .map(|rules| &rules.common)
                .and_then(|rules| {
                    rules
                        .const_value
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::Const {
                    expected: const_value.to_string(),
                    actual: value.to_string(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(enumerate) = &float_schema.enumerate {
        if !enumerate.contains(&value) {
            let level = value_rules
                .map(|rules| &rules.common)
                .and_then(|rules| {
                    rules
                        .enumerate
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::Enumerate {
                    expected: enumerate.iter().map(ToString::to_string).collect(),
                    actual: value.to_string(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(maximum) = &float_schema.maximum {
        if value > *maximum {
            let level = value_rules
                .map(|rules| &rules.value)
                .and_then(|rules| {
                    rules
                        .integer_maximum
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerMaximum {
                    maximum: *maximum as i64,
                    actual: value as i64,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(minimum) = &float_schema.minimum {
        if value < *minimum {
            let level = value_rules
                .map(|rules| &rules.value)
                .and_then(|rules| {
                    rules
                        .integer_minimum
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerMinimum {
                    minimum: *minimum as i64,
                    actual: value as i64,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(exclusive_maximum) = &float_schema.exclusive_maximum {
        if value >= *exclusive_maximum {
            let level = value_rules
                .map(|rules| &rules.value)
                .and_then(|rules| {
                    rules
                        .integer_exclusive_maximum
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerExclusiveMaximum {
                    maximum: (*exclusive_maximum as i64) - 1,
                    actual: value as i64,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(exclusive_minimum) = &float_schema.exclusive_minimum {
        if value <= *exclusive_minimum {
            let level = value_rules
                .map(|rules| &rules.value)
                .and_then(|rules| {
                    rules
                        .integer_exclusive_minimum
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerExclusiveMinimum {
                    minimum: (*exclusive_minimum as i64) + 1,
                    actual: value as i64,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(multiple_of) = &float_schema.multiple_of {
        if value % *multiple_of != 0.0 {
            let level = value_rules
                .map(|rules| &rules.value)
                .and_then(|rules| {
                    rules
                        .integer_multiple_of
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::IntegerMultipleOf {
                    multiple_of: *multiple_of as i64,
                    actual: value as i64,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        if float_schema.deprecated == Some(true) {
            let level = value_rules
                .map(|rules| &rules.common)
                .and_then(|rules| {
                    rules
                        .deprecated
                        .as_ref()
                        .map(SeverityLevelDefaultWarn::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::from(accessors),
                    value.to_string(),
                )),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
