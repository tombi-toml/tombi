use itertools::Itertools;
use tombi_ast::TombiValueCommentDirective;
use tombi_comment_directive::value::{StringCommonFormatRules, StringCommonLintRules};
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_regex::Regex;
use tombi_schema_store::ValueSchema;
use tombi_severity_level::SeverityLevelDefaultError;
use tombi_x_keyword::StringFormat;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    comment_directive::get_tombi_key_value_rules_and_diagnostics,
    validate::{
        format, handle_deprecated_value, handle_type_mismatch, handle_unused_noqa,
        validate_adjacent_applicators,
    },
};

use super::{Validate, validate_all_of, validate_any_of, validate_one_of};

impl Validate for tombi_document_tree::String {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), crate::Error>> {
        async move {
            let (lint_rules, lint_rules_diagnostics) =
                get_tombi_key_value_rules_and_diagnostics::<
                    StringCommonFormatRules,
                    StringCommonLintRules,
                >(self.comment_directives(), accessors)
                .await;

            let result = if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::String(string_schema) => {
                        let format_assertion = schema_context
                            .root_schema
                            .is_none_or(|root| root.format_assertion())
                            || string_schema
                                .format
                                .is_some_and(|f| schema_context.has_string_format(f));
                        validate_string(
                            self,
                            accessors,
                            string_schema,
                            current_schema,
                            schema_context,
                            self.comment_directives()
                                .map(|directives| directives.cloned().collect_vec())
                                .as_deref(),
                            format_assertion,
                            lint_rules.as_ref(),
                        )
                        .await
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            self.comment_directives()
                                .map(|directives| directives.cloned().collect_vec())
                                .as_deref(),
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
                            self.comment_directives()
                                .map(|directives| directives.cloned().collect_vec())
                                .as_deref(),
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
                            self.comment_directives()
                                .map(|directives| directives.cloned().collect_vec())
                                .as_deref(),
                            lint_rules.as_ref().map(|rules| &rules.common),
                        )
                        .await
                    }
                    ValueSchema::Null => return Ok(()),
                    // When the schema expects a TOML date/time type but the value is a string,
                    // check if x-tombi-string-formats includes the corresponding format.
                    // If so, validate the string against the format instead of reporting type mismatch.
                    ValueSchema::OffsetDateTime(_) => validate_string_as_date_format(
                        self,
                        StringFormat::DateTime,
                        tombi_schema_store::ValueType::OffsetDateTime,
                        format::date_time::validate_format,
                        schema_context,
                        lint_rules.as_ref(),
                    ),
                    ValueSchema::LocalDateTime(_) => validate_string_as_date_format(
                        self,
                        StringFormat::DateTimeLocal,
                        tombi_schema_store::ValueType::LocalDateTime,
                        format::local_date_time::validate_format,
                        schema_context,
                        lint_rules.as_ref(),
                    ),
                    ValueSchema::LocalDate(_) => validate_string_as_date_format(
                        self,
                        StringFormat::Date,
                        tombi_schema_store::ValueType::LocalDate,
                        format::date::validate_format,
                        schema_context,
                        lint_rules.as_ref(),
                    ),
                    ValueSchema::LocalTime(_) => validate_string_as_date_format(
                        self,
                        StringFormat::TimeLocal,
                        tombi_schema_store::ValueType::LocalTime,
                        format::local_time::validate_format,
                        schema_context,
                        lint_rules.as_ref(),
                    ),
                    value_schema => handle_type_mismatch(
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
    current_schema: &tombi_schema_store::CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    comment_directives: Option<&[TombiValueCommentDirective]>,
    format_assertion: bool,
    lint_rules: Option<&StringCommonLintRules>,
) -> Result<(), crate::Error> {
    let mut diagnostics = vec![];

    if let Err(crate::Error {
        diagnostics: base_diagnostics,
        ..
    }) = validate_raw_string(
        string_value.value(),
        &string_value.to_string(),
        string_value.range(),
        string_schema,
        format_assertion,
        lint_rules,
        string_value.comment_directives(),
    ) {
        diagnostics.extend(base_diagnostics);
    }

    if diagnostics.is_empty() {
        handle_deprecated_value(
            &mut diagnostics,
            string_schema.deprecated,
            accessors,
            string_value,
            string_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
        );
    }

    let base_result = if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.into())
    };

    crate::validate::merge_validation_results(
        base_result,
        validate_adjacent_applicators(
            string_value,
            accessors,
            string_schema.one_of.as_deref(),
            string_schema.any_of.as_deref(),
            string_schema.all_of.as_deref(),
            string_schema.not.as_deref(),
            current_schema,
            schema_context,
            comment_directives,
            lint_rules.map(|rules| &rules.common),
        )
        .await,
    )
}

/// Validate a string value against a `StringSchema`.
///
/// This is the core string schema validation logic shared between
/// string value validation and `propertyNames` validation in tables.
///
/// When `lint_rules` is `None`, all checks use default error severity
/// and noqa handling is skipped.
pub(crate) fn validate_raw_string<'a>(
    value: &str,
    display_value: &str,
    range: tombi_text::Range,
    string_schema: &tombi_schema_store::StringSchema,
    format_assertion: bool,
    lint_rules: Option<&StringCommonLintRules>,
    comment_directives: Option<impl IntoIterator<Item = &'a TombiValueCommentDirective> + 'a>,
) -> Result<(), crate::Error> {
    let mut diagnostics = vec![];

    let comment_directives =
        comment_directives.map(|directives| directives.into_iter().cloned().collect_vec());

    if let Some(const_value) = &string_schema.const_value
        && value != const_value.as_str()
    {
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
                actual: display_value.to_string(),
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.common.const_value.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            comment_directives.as_ref(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "const-value",
        );
    }

    if let Some(r#enum) = &string_schema.r#enum
        && !r#enum.iter().any(|item| item == value)
    {
        let level = lint_rules
            .map(|rules| &rules.common)
            .and_then(|rules| rules.r#enum().map(SeverityLevelDefaultError::from))
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::Enum {
                expected: r#enum.iter().map(|s| format!("\"{s}\"")).collect(),
                actual: display_value.to_string(),
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.common.r#enum())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            comment_directives.as_ref(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "enum",
        );
    }

    let length = UnicodeSegmentation::graphemes(value, true).count();

    if let Some(max_length) = &string_schema.max_length
        && length > *max_length
    {
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
    } else if lint_rules
        .and_then(|rules| rules.value.string_max_length.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            comment_directives.as_ref(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "string-max-length",
        );
    }

    if let Some(min_length) = &string_schema.min_length
        && length < *min_length
    {
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
    } else if lint_rules
        .and_then(|rules| rules.value.string_min_length.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            comment_directives.as_ref(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "string-min-length",
        );
    }

    if format_assertion
        && let Some(format) = string_schema.format
        && !match format {
            StringFormat::Email => format::email::validate_format(value),
            StringFormat::Hostname => format::hostname::validate_format(value),
            StringFormat::Uri => format::uri::validate_format(value),
            StringFormat::UriReference => format::uri_reference::validate_format(value),
            StringFormat::Uuid => format::uuid::validate_format(value),
            StringFormat::Ipv4 => format::ipv4::validate_format(value),
            StringFormat::Ipv6 => format::ipv6::validate_format(value),
            StringFormat::DateTime => format::date_time::validate_format(value),
            StringFormat::DateTimeLocal => format::local_date_time::validate_format(value),
            StringFormat::Date => format::date::validate_format(value),
            StringFormat::Time => format::time::validate_format(value),
            StringFormat::TimeLocal => format::local_time::validate_format(value),
            StringFormat::Regex => format::regex::validate_format(value),
            StringFormat::JsonPointer => format::json_pointer::validate_format(value),
        }
    {
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
                actual: display_value.to_string(),
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.string_format.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            comment_directives.as_ref(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "string-format",
        );
    }

    if let Some(pattern) = &string_schema.pattern
        && let Ok(regex) =
            Regex::new(pattern).inspect_err(|_| log::warn!("Invalid regex pattern: {:?}", pattern))
        && !regex.is_match(value)
    {
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
                actual: display_value.to_string(),
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.string_pattern.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            comment_directives.as_ref(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "string-pattern",
        );
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.into())
    }
}

/// When a string value encounters a TOML date/time schema,
/// check if x-tombi-string-formats includes the corresponding format.
/// If so, validate the string against the format; otherwise, report type mismatch.
fn validate_string_as_date_format(
    string_value: &tombi_document_tree::String,
    string_format: StringFormat,
    expected_value_type: tombi_schema_store::ValueType,
    validate_fn: fn(&str) -> bool,
    schema_context: &tombi_schema_store::SchemaContext,
    lint_rules: Option<&StringCommonLintRules>,
) -> Result<(), crate::Error> {
    if !schema_context.has_string_format(string_format) {
        return handle_type_mismatch(
            expected_value_type,
            string_value.value_type(),
            string_value.range(),
            lint_rules.map(|rules| &rules.common),
        );
    }

    if validate_fn(string_value.value()) {
        Ok(())
    } else {
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .string_format
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        let mut diagnostics = vec![];
        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::StringFormat {
                format: string_format,
                actual: string_value.to_string(),
            }),
            range: string_value.range(),
        }
        .push_diagnostic_with_level(level, &mut diagnostics);

        Err(diagnostics.into())
    }
}
