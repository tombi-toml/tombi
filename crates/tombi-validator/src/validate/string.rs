use regex::Regex;
use tombi_comment_directive::value::{StringCommonFormatRules, StringCommonLintRules};
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;
use tombi_severity_level::SeverityLevelDefaultError;
use tombi_x_keyword::StringFormat;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    comment_directive::get_tombi_key_table_value_rules_and_diagnostics,
    validate::{format, push_deprecated_value, type_mismatch},
};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::String {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), crate::Error>> {
        async move {
            let (lint_rules, lint_rules_diagnostics) =
                if let Some(comment_directives) = self.comment_directives() {
                    get_tombi_key_table_value_rules_and_diagnostics::<
                        StringCommonFormatRules,
                        StringCommonLintRules,
                    >(comment_directives, accessors)
                    .await
                } else {
                    (None, Vec::with_capacity(0))
                };

            let result = if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::String(string_schema) => {
                        validate_string(self, accessors, string_schema, lint_rules.as_ref()).await
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            lint_rules.as_ref().map(|rules| &rules.common),
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        validate_any_of(
                            self,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                            lint_rules.as_ref().map(|rules| &rules.common),
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        validate_all_of(
                            self,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                            lint_rules.as_ref().map(|rules| &rules.common),
                        )
                        .await
                    }
                    ValueSchema::Null => return Ok(()),
                    value_schema => type_mismatch(
                        value_schema.value_type().await,
                        self.value_type(),
                        self.range(),
                        lint_rules.as_ref().map(|rules| &rules.common),
                    ),
                }
            } else {
                Ok(())
            };

            match result {
                Ok(()) => {
                    if lint_rules_diagnostics.is_empty() {
                        Ok(())
                    } else {
                        Err(lint_rules_diagnostics.into())
                    }
                }
                Err(mut error) => {
                    error.prepend_diagnostics(lint_rules_diagnostics);
                    Err(error)
                }
            }
        }
        .boxed()
    }
}

async fn validate_string(
    string_value: &tombi_document_tree::String,
    accessors: &[tombi_schema_store::Accessor],
    string_schema: &tombi_schema_store::StringSchema,
    lint_rules: Option<&StringCommonLintRules>,
) -> Result<(), crate::Error> {
    let mut diagnostics = vec![];
    let value = string_value.value().to_string();
    let range = string_value.range();

    if let Some(const_value) = &string_schema.const_value {
        if value != *const_value {
            let level = lint_rules
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
                    expected: format!("\"{const_value}\""),
                    actual: string_value.to_string(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(enumerate) = &string_schema.enumerate {
        if !enumerate.contains(&value) {
            let level = lint_rules
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
                    expected: enumerate.iter().map(|s| format!("\"{s}\"")).collect(),
                    actual: string_value.to_string(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    let length = UnicodeSegmentation::graphemes(value.as_str(), true).count();

    if let Some(max_length) = &string_schema.max_length {
        if length > *max_length {
            let level = lint_rules
                .map(|rules| &rules.value)
                .and_then(|rules| {
                    rules
                        .string_max_length
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::StringMaxLength {
                    maximum: *max_length,
                    actual: length,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(min_length) = &string_schema.min_length {
        if length < *min_length {
            let level = lint_rules
                .map(|rules| &rules.value)
                .and_then(|rules| {
                    rules
                        .string_min_length
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::StringMinLength {
                    minimum: *min_length,
                    actual: length,
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(format) = string_schema.format {
        match format {
            StringFormat::Email => {
                if !format::email::validate_format(&value) {
                    let level = lint_rules
                        .map(|rules| &rules.value)
                        .and_then(|rules| {
                            rules
                                .string_format
                                .as_ref()
                                .map(SeverityLevelDefaultError::from)
                        })
                        .unwrap_or_default();

                    crate::Diagnostic {
                        kind: Box::new(crate::DiagnosticKind::StringFormat {
                            format,
                            actual: string_value.to_string(),
                        }),
                        range,
                    }
                    .push_diagnostic_with_level(level, &mut diagnostics);
                }
            }
            StringFormat::Hostname => {
                if !format::hostname::validate_format(&value) {
                    let level = lint_rules
                        .map(|rules| &rules.value)
                        .and_then(|rules| {
                            rules
                                .string_format
                                .as_ref()
                                .map(SeverityLevelDefaultError::from)
                        })
                        .unwrap_or_default();

                    crate::Diagnostic {
                        kind: Box::new(crate::DiagnosticKind::StringFormat {
                            format,
                            actual: string_value.to_string(),
                        }),
                        range,
                    }
                    .push_diagnostic_with_level(level, &mut diagnostics);
                }
            }
            StringFormat::Uri => {
                if !format::uri::validate_format(&value) {
                    let level = lint_rules
                        .map(|rules| &rules.value)
                        .and_then(|rules| {
                            rules
                                .string_format
                                .as_ref()
                                .map(SeverityLevelDefaultError::from)
                        })
                        .unwrap_or_default();

                    crate::Diagnostic {
                        kind: Box::new(crate::DiagnosticKind::StringFormat {
                            format,
                            actual: string_value.to_string(),
                        }),
                        range,
                    }
                    .push_diagnostic_with_level(level, &mut diagnostics);
                }
            }
            StringFormat::Uuid => {
                if !format::uuid::validate_format(&value) {
                    let level = lint_rules
                        .map(|rules| &rules.value)
                        .and_then(|rules| {
                            rules
                                .string_format
                                .as_ref()
                                .map(SeverityLevelDefaultError::from)
                        })
                        .unwrap_or_default();

                    crate::Diagnostic {
                        kind: Box::new(crate::DiagnosticKind::StringFormat {
                            format,
                            actual: string_value.to_string(),
                        }),
                        range,
                    }
                    .push_diagnostic_with_level(level, &mut diagnostics);
                }
            }
        };
    }

    if let Some(pattern) = &string_schema.pattern {
        if let Ok(regex) = Regex::new(pattern) {
            if !regex.is_match(&value) {
                let level = lint_rules
                    .map(|rules| &rules.value)
                    .and_then(|rules| {
                        rules
                            .string_pattern
                            .as_ref()
                            .map(SeverityLevelDefaultError::from)
                    })
                    .unwrap_or_default();

                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::StringPattern {
                        pattern: pattern.clone(),
                        actual: string_value.to_string(),
                    }),
                    range,
                }
                .push_diagnostic_with_level(level, &mut diagnostics);
            }
        } else {
            tracing::warn!("Invalid regex pattern: {:?}", pattern);
        }
    }

    if diagnostics.is_empty() && string_schema.deprecated == Some(true) {
        push_deprecated_value(
            &mut diagnostics,
            accessors,
            string_value,
            lint_rules.as_ref().map(|rules| &rules.common),
        );
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.into())
    }
}
