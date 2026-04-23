use itertools::Itertools;
use tombi_ast::TombiValueCommentDirective;
use tombi_comment_directive::value::KeyLinkRules;
use tombi_comment_directive::value::{StringCommonFormatRules, StringCommonLintRules};
use tombi_document_tree::{LikeString, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_regex::Regex;
use tombi_schema_store::ValueSchema;
use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};
use tombi_x_keyword::StringFormat;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    comment_directive::{
        get_tombi_key_rules_and_diagnostics, get_tombi_key_table_value_rules_and_diagnostics,
    },
    validate::{
        format, handle_anything_schema, handle_deprecated_value, handle_nothing_schema,
        handle_type_mismatch, handle_unused_noqa, validate_adjacent_applicators,
    },
};

use super::{Validate, validate_all_of, validate_any_of, validate_one_of};

impl Validate for tombi_document_tree::String {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<crate::EvaluatedLocations, crate::Error>> {
        validate_like_string(self, accessors, current_schema, schema_context, false)
    }
}

impl Validate for tombi_document_tree::Key {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<crate::EvaluatedLocations, crate::Error>> {
        validate_like_string(self, accessors, current_schema, schema_context, true)
    }
}

fn validate_like_string<'a: 'b, 'b, T>(
    string_value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
    schema_context: &'a tombi_schema_store::SchemaContext,
    enable_key_empty_validation: bool,
) -> BoxFuture<'b, Result<crate::EvaluatedLocations, crate::Error>>
where
    T: Validate + LikeString + ValueImpl + ToString + Sync + Send + std::fmt::Debug,
{
    async move {
        let comment_directives = string_value
            .comment_directives()
            .map(|directives| directives.cloned().collect_vec());

        let (key_rules, key_rules_diagnostics) = if enable_key_empty_validation {
            let (rules, diagnostics) = get_tombi_key_rules_and_diagnostics(
                comment_directives
                    .as_deref()
                    .map(|directives| directives.iter()),
            )
            .await;
            (rules.map(|rules| rules.value), diagnostics)
        } else {
            (None, vec![])
        };

        let (lint_rules, lint_rules_diagnostics) =
            if !enable_key_empty_validation || current_schema.is_some() {
                get_tombi_key_table_value_rules_and_diagnostics::<
                    StringCommonFormatRules,
                    StringCommonLintRules,
                >(
                    comment_directives
                        .as_deref()
                        .map(|directives| directives.iter()),
                    accessors,
                )
                .await
            } else {
                (None, vec![])
            };

        let result = if let Some(current_schema) = current_schema {
            match current_schema.value_schema.as_ref() {
                ValueSchema::String(string_schema) => {
                    let key_empty_result = if enable_key_empty_validation
                        && should_validate_key_empty(Some(current_schema))
                    {
                        validate_key_empty_rule(string_value, key_rules.as_ref())
                    } else {
                        Ok(crate::EvaluatedLocations::new())
                    };
                    let format_assertion = schema_context
                        .root_schema
                        .is_none_or(|root| root.format_assertion())
                        || string_schema
                            .format
                            .is_some_and(|f| schema_context.has_string_format(f));
                    crate::validate::merge_validation_results(
                        key_empty_result,
                        validate_string(
                            string_value,
                            accessors,
                            string_schema,
                            current_schema,
                            schema_context,
                            comment_directives.as_deref(),
                            format_assertion,
                            lint_rules.as_ref(),
                        )
                        .await,
                    )
                }
                ValueSchema::OneOf(one_of_schema) => {
                    let result = validate_one_of(
                        string_value,
                        accessors,
                        one_of_schema,
                        current_schema,
                        schema_context,
                        comment_directives.as_deref(),
                        lint_rules.as_ref().map(|rules| &rules.common),
                    )
                    .await;
                    if enable_key_empty_validation
                        && should_validate_key_empty(Some(current_schema))
                    {
                        crate::validate::merge_validation_results(
                            result,
                            validate_key_empty_rule(string_value, key_rules.as_ref()),
                        )
                    } else {
                        result
                    }
                }
                ValueSchema::AnyOf(any_of_schema) => {
                    let result = validate_any_of(
                        string_value,
                        accessors,
                        any_of_schema,
                        current_schema,
                        schema_context,
                        comment_directives.as_deref(),
                        lint_rules.as_ref().map(|rules| &rules.common),
                    )
                    .await;
                    if enable_key_empty_validation
                        && should_validate_key_empty(Some(current_schema))
                    {
                        crate::validate::merge_validation_results(
                            result,
                            validate_key_empty_rule(string_value, key_rules.as_ref()),
                        )
                    } else {
                        result
                    }
                }
                ValueSchema::AllOf(all_of_schema) => {
                    let result = validate_all_of(
                        string_value,
                        accessors,
                        all_of_schema,
                        current_schema,
                        schema_context,
                        comment_directives.as_deref(),
                        lint_rules.as_ref().map(|rules| &rules.common),
                    )
                    .await;
                    if enable_key_empty_validation
                        && should_validate_key_empty(Some(current_schema))
                    {
                        crate::validate::merge_validation_results(
                            result,
                            validate_key_empty_rule(string_value, key_rules.as_ref()),
                        )
                    } else {
                        result
                    }
                }
                ValueSchema::Null => return Ok(crate::EvaluatedLocations::new()),
                ValueSchema::Anything(_) => handle_anything_schema(string_value),
                ValueSchema::Nothing(_) => handle_nothing_schema(string_value),
                // When the schema expects a TOML date/time type but the value is a string,
                // check if x-tombi-string-formats includes the corresponding format.
                // If so, validate the string against the format instead of reporting type mismatch.
                ValueSchema::OffsetDateTime(_) => validate_string_as_date_format(
                    string_value,
                    StringFormat::DateTime,
                    tombi_schema_store::ValueType::OffsetDateTime,
                    format::date_time::validate_format,
                    schema_context,
                    lint_rules.as_ref(),
                ),
                ValueSchema::LocalDateTime(_) => validate_string_as_date_format(
                    string_value,
                    StringFormat::DateTimeLocal,
                    tombi_schema_store::ValueType::LocalDateTime,
                    format::local_date_time::validate_format,
                    schema_context,
                    lint_rules.as_ref(),
                ),
                ValueSchema::LocalDate(_) => validate_string_as_date_format(
                    string_value,
                    StringFormat::Date,
                    tombi_schema_store::ValueType::LocalDate,
                    format::date::validate_format,
                    schema_context,
                    lint_rules.as_ref(),
                ),
                ValueSchema::LocalTime(_) => validate_string_as_date_format(
                    string_value,
                    StringFormat::TimeLocal,
                    tombi_schema_store::ValueType::LocalTime,
                    format::local_time::validate_format,
                    schema_context,
                    lint_rules.as_ref(),
                ),
                value_schema => handle_type_mismatch(
                    value_schema.value_type().await,
                    string_value.value_type(),
                    ValueImpl::range(string_value),
                    lint_rules.as_ref().map(|rules| &rules.common),
                ),
            }
        } else if enable_key_empty_validation && should_validate_key_empty(None) {
            validate_key_empty_rule(string_value, key_rules.as_ref())
        } else {
            Ok(crate::EvaluatedLocations::new())
        };

        match result {
            Ok(result) => {
                let mut total_diagnostics = key_rules_diagnostics;
                total_diagnostics.extend(lint_rules_diagnostics);

                if total_diagnostics.is_empty() {
                    Ok(result)
                } else {
                    Err(total_diagnostics.into())
                }
            }
            Err(mut error) => {
                error.prepend_diagnostics(lint_rules_diagnostics);
                error.prepend_diagnostics(key_rules_diagnostics);
                Err(error)
            }
        }
    }
    .boxed()
}

fn should_validate_key_empty(
    current_schema: Option<&tombi_schema_store::CurrentSchema<'_>>,
) -> bool {
    !matches!(
        current_schema.map(|schema| schema.value_schema.as_ref()),
        Some(ValueSchema::String(string_schema)) if string_schema.min_length == Some(0)
    )
}

#[allow(clippy::result_large_err)]
fn validate_key_empty_rule<T>(
    string_value: &T,
    key_rules: Option<&KeyLinkRules>,
) -> Result<crate::EvaluatedLocations, crate::Error>
where
    T: LikeString + ValueImpl,
{
    if !string_value.value().is_empty() {
        return Ok(crate::EvaluatedLocations::new());
    }

    let level = key_rules
        .and_then(|rules| rules.key_empty.as_ref().map(SeverityLevelDefaultWarn::from))
        .unwrap_or_default();

    let mut diagnostics = vec![];
    crate::Diagnostic {
        kind: Box::new(crate::DiagnosticKind::KeyEmpty),
        range: ValueImpl::range(string_value),
    }
    .push_diagnostic_with_level(level, &mut diagnostics);

    Err(diagnostics.into())
}

async fn validate_string<T>(
    string_value: &T,
    accessors: &[tombi_schema_store::Accessor],
    string_schema: &tombi_schema_store::StringSchema,
    current_schema: &tombi_schema_store::CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    comment_directives: Option<&[TombiValueCommentDirective]>,
    format_assertion: bool,
    lint_rules: Option<&StringCommonLintRules>,
) -> Result<crate::EvaluatedLocations, crate::Error>
where
    T: LikeString + ValueImpl + ToString + Validate + Sync + Send + std::fmt::Debug,
{
    let result = validate_raw_string(
        string_value.value(),
        &string_value.to_string(),
        ValueImpl::range(string_value),
        string_schema,
        format_assertion,
        lint_rules,
        string_value.comment_directives(),
    );

    let base_result = match result {
        Ok(result) => {
            let mut diagnostics = vec![];

            handle_deprecated_value(
                &mut diagnostics,
                string_schema.deprecated,
                accessors,
                string_value,
                Some(current_schema),
                schema_context,
                string_value.comment_directives(),
                lint_rules.as_ref().map(|rules| &rules.common),
            );

            if diagnostics.is_empty() {
                Ok(result)
            } else {
                Err(diagnostics.into())
            }
        }
        Err(error) => Err(error),
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
#[allow(clippy::result_large_err)]
pub(crate) fn validate_raw_string<'a>(
    value: &str,
    display_value: &str,
    range: tombi_text::Range,
    string_schema: &tombi_schema_store::StringSchema,
    format_assertion: bool,
    lint_rules: Option<&StringCommonLintRules>,
    comment_directives: Option<impl IntoIterator<Item = &'a TombiValueCommentDirective> + 'a>,
) -> Result<crate::EvaluatedLocations, crate::Error> {
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
        Ok(crate::EvaluatedLocations::new())
    } else {
        Err(diagnostics.into())
    }
}

/// When a string value encounters a TOML date/time schema,
/// check if x-tombi-string-formats includes the corresponding format.
/// If so, validate the string against the format; otherwise, report type mismatch.
#[allow(clippy::result_large_err)]
fn validate_string_as_date_format(
    string_value: &(impl LikeString + ValueImpl + ToString),
    string_format: StringFormat,
    expected_value_type: tombi_schema_store::ValueType,
    validate_fn: fn(&str) -> bool,
    schema_context: &tombi_schema_store::SchemaContext,
    lint_rules: Option<&StringCommonLintRules>,
) -> Result<crate::EvaluatedLocations, crate::Error> {
    if !schema_context.has_string_format(string_format) {
        return handle_type_mismatch(
            expected_value_type,
            string_value.value_type(),
            ValueImpl::range(string_value),
            lint_rules.map(|rules| &rules.common),
        );
    }

    if validate_fn(string_value.value()) {
        Ok(crate::EvaluatedLocations::new())
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
            range: ValueImpl::range(string_value),
        }
        .push_diagnostic_with_level(level, &mut diagnostics);

        Err(diagnostics.into())
    }
}
