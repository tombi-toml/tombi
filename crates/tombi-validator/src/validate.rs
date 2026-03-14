mod all_of;
mod any_of;
mod array;
mod boolean;
mod float;
mod if_then_else;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod not_schema;
mod offset_date_time;
mod one_of;
mod string;
mod table;
mod value;

pub mod format {
    pub mod date;
    pub mod date_time;
    pub mod email;
    pub mod hostname;
    pub mod ipv4;
    pub mod ipv6;
    pub mod json_pointer;
    pub mod local_date_time;
    pub mod local_time;
    pub mod regex;
    pub mod time;
    pub mod uri;
    pub mod uri_reference;
    pub mod uuid;
}

use std::borrow::Cow;

pub use all_of::validate_all_of;
pub use any_of::validate_any_of;
use itertools::Itertools;
pub use one_of::validate_one_of;
use tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
use tombi_document_tree::{TryIntoDocumentTree, dig_keys};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::CurrentSchema;
use tombi_severity_level::{SeverityLevel, SeverityLevelDefaultError, SeverityLevelDefaultWarn};
use tombi_text::RelativePosition;

pub fn validate<'a: 'b, 'b>(
    tree: tombi_document_tree::DocumentTree,
    source_schema: Option<&'a tombi_schema_store::SourceSchema>,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
    async move {
        let current_schema = source_schema.as_ref().and_then(|source_schema| {
            source_schema
                .root_schema
                .as_deref()
                .and_then(|root_schema| {
                    root_schema
                        .value_schema
                        .as_ref()
                        .map(|value_schema| CurrentSchema {
                            value_schema: value_schema.clone(),
                            schema_uri: Cow::Borrowed(&root_schema.schema_uri),
                            definitions: Cow::Borrowed(&root_schema.definitions),
                        })
                })
        });

        if let Err(crate::Error { diagnostics, .. }) = tree
            .validate(&[], current_schema.as_ref(), schema_context)
            .await
        {
            Err(diagnostics.into_iter().unique().collect_vec())
        } else {
            Ok(())
        }
    }
    .boxed()
}

pub trait Validate {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), crate::Error>>;
}

pub fn handle_deprecated<'a, T>(
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
    deprecated: Option<bool>,
    accessors: &[tombi_schema_store::Accessor],
    value: &T,
    comment_directives: Option<
        impl IntoIterator<Item = &'a tombi_ast::TombiValueCommentDirective> + 'a,
    >,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) where
    T: tombi_document_tree::ValueImpl,
{
    if deprecated == Some(true) {
        let level = common_rules
            .and_then(|rules| {
                rules
                    .deprecated
                    .as_ref()
                    .map(SeverityLevelDefaultWarn::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::Deprecated(
                tombi_schema_store::SchemaAccessors::from(accessors),
            )),
            range: value.range(),
        }
        .push_diagnostic_with_level(level, diagnostics);
    } else if common_rules
        .and_then(|rules| rules.deprecated.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(diagnostics, comment_directives, common_rules, "deprecated");
    }
}

pub fn handle_deprecated_value<'a, T>(
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
    deprecated: Option<bool>,
    accessors: &[tombi_schema_store::Accessor],
    value: &T,
    comment_directives: Option<
        impl IntoIterator<Item = &'a tombi_ast::TombiValueCommentDirective> + 'a,
    >,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) where
    T: tombi_document_tree::ValueImpl + ToString,
{
    if deprecated == Some(true) {
        let level = common_rules
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
            range: value.range(),
        }
        .push_diagnostic_with_level(level, diagnostics);
    } else if common_rules
        .and_then(|rules| rules.deprecated.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(diagnostics, comment_directives, common_rules, "deprecated");
    }
}

fn handle_type_mismatch(
    expected: tombi_schema_store::ValueType,
    actual: tombi_document_tree::ValueType,
    range: tombi_text::Range,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) -> Result<(), crate::Error> {
    let mut diagnostics = vec![];

    let level = common_rules
        .and_then(|common_rules| {
            common_rules
                .type_mismatch
                .as_ref()
                .map(SeverityLevelDefaultError::from)
        })
        .unwrap_or_default();

    crate::Diagnostic {
        kind: Box::new(crate::DiagnosticKind::TypeMismatch { expected, actual }),
        range,
    }
    .push_diagnostic_with_level(level, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(crate::Error {
            score: 0,
            diagnostics,
        })
    }
}

#[inline]
pub(crate) fn handle_anything_schema<T>(_value: &T) -> Result<(), crate::Error>
where
    T: tombi_document_tree::ValueImpl,
{
    Ok(())
}

pub(crate) fn handle_nothing_schema<T>(value: &T) -> Result<(), crate::Error>
where
    T: tombi_document_tree::ValueImpl,
{
    let mut diagnostics = vec![];
    crate::Diagnostic {
        kind: Box::new(crate::DiagnosticKind::Nothing),
        range: value.range(),
    }
    .push_diagnostic_with_level(SeverityLevelDefaultError::default(), &mut diagnostics);
    Err(diagnostics.into())
}

fn handle_unused_noqa<'a>(
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
    comment_directives: Option<
        impl IntoIterator<Item = &'a tombi_ast::TombiValueCommentDirective> + 'a,
    >,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
    rule_name: &'static str,
) {
    let Some(comment_directives) = comment_directives else {
        return;
    };

    if common_rules
        .and_then(|rules| rules.unused_noqa.as_ref())
        .and_then(|rules| rules.disabled)
        .unwrap_or(false)
    {
        return;
    }

    for tombi_ast::TombiValueCommentDirective {
        content,
        content_range,
        ..
    } in comment_directives
    {
        let Ok(root) = tombi_parser::parse(content).try_into_root() else {
            continue;
        };

        let Ok(document_tree) = root.try_into_document_tree(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
        else {
            continue;
        };

        if let Some((key, value)) =
            dig_keys(&document_tree, &["lint", "rules", rule_name, "disabled"])
        {
            let range = key.range() + value.range();
            let range = tombi_text::Range::new(
                content_range.start + RelativePosition::from(range.start),
                content_range.start + RelativePosition::from(range.end),
            );
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::UnusedNoqa { rule_name }),
                range,
            }
            .push_diagnostic_with_level(SeverityLevel::Warn, diagnostics);
            return;
        }
    }
}

fn has_error_level_diagnostics(error: &crate::Error) -> bool {
    error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.level() == tombi_diagnostic::Level::ERROR)
}

pub(crate) fn is_assertion_success(result: &Result<(), crate::Error>) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => !has_error_level_diagnostics(error),
    }
}

fn is_multiple_of_with_tolerance(value: f64, multiple_of: f64) -> bool {
    if !value.is_finite() || !multiple_of.is_finite() {
        return false;
    }
    if multiple_of <= 0.0 {
        return true;
    }

    let quotient = value / multiple_of;
    let nearest = quotient.round();
    let tolerance = f64::EPSILON * quotient.abs().max(1.0) * 8.0;
    (quotient - nearest).abs() <= tolerance
}

fn validate_deprecated<'a, T>(
    deprecated: Option<bool>,
    accessors: &[tombi_schema_store::Accessor],
    value: &T,
    comment_directives: Option<
        impl IntoIterator<Item = &'a tombi_ast::TombiValueCommentDirective> + 'a,
    >,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) -> Result<(), crate::Error>
where
    T: tombi_document_tree::ValueImpl,
{
    let mut diagnostics = Vec::with_capacity(1);
    handle_deprecated(
        &mut diagnostics,
        deprecated,
        accessors,
        value,
        comment_directives,
        common_rules,
    );

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.into())
    }
}

fn merge_validation_results(
    primary: Result<(), crate::Error>,
    secondary: Result<(), crate::Error>,
) -> Result<(), crate::Error> {
    match (primary, secondary) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(mut left), Err(right)) => {
            left.score = left.score.max(right.score);
            left.diagnostics.extend(right.diagnostics);
            Err(left)
        }
    }
}

pub(crate) fn filter_table_strict_additional_diagnostics(
    mut error: crate::Error,
) -> Result<(), crate::Error> {
    error
        .diagnostics
        .retain(|diagnostic| diagnostic.code() != "table-strict-additional-keys");

    if error.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(error)
    }
}

pub fn validate_adjacent_applicators<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    one_of_schema: Option<&'a tombi_schema_store::OneOfSchema>,
    any_of_schema: Option<&'a tombi_schema_store::AnyOfSchema>,
    all_of_schema: Option<&'a tombi_schema_store::AllOfSchema>,
    not_schema: Option<&'a tombi_schema_store::NotSchema>,
    current_schema: &'a tombi_schema_store::CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_directives: Option<&'a [tombi_ast::TombiValueCommentDirective]>,
    common_rules: Option<&'a tombi_comment_directive::value::CommonLintRules>,
) -> BoxFuture<'b, Result<(), crate::Error>>
where
    T: Validate + tombi_document_tree::ValueImpl + Sync + Send + std::fmt::Debug,
{
    async move {
        if one_of_schema.is_none()
            && any_of_schema.is_none()
            && all_of_schema.is_none()
            && not_schema.is_none()
        {
            return Ok(());
        }

        let mut result = Ok(());

        if let Some(one_of_schema) = one_of_schema {
            let adjacent_result = validate_one_of(
                value,
                accessors,
                one_of_schema,
                current_schema,
                schema_context,
                comment_directives,
                common_rules,
            )
            .await;
            result = merge_validation_results(result, adjacent_result);
        }
        if let Some(any_of_schema) = any_of_schema {
            let adjacent_result = validate_any_of(
                value,
                accessors,
                any_of_schema,
                current_schema,
                schema_context,
                comment_directives,
                common_rules,
            )
            .await;
            result = merge_validation_results(result, adjacent_result);
        }
        if let Some(all_of_schema) = all_of_schema {
            let adjacent_result = validate_all_of(
                value,
                accessors,
                all_of_schema,
                current_schema,
                schema_context,
                comment_directives,
                common_rules,
            )
            .await;
            result = merge_validation_results(result, adjacent_result);
        }
        if let Some(not_schema) = not_schema {
            result = merge_validation_results(
                result,
                not_schema::validate_not(
                    value,
                    accessors,
                    not_schema,
                    current_schema,
                    schema_context,
                    comment_directives.map(|directives| directives.iter()),
                    common_rules,
                )
                .await,
            );
        }

        result
    }
    .boxed()
}

pub fn validate_resolved_schema<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    resolved_schema: &'a tombi_schema_store::CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_directives: Option<&'a [tombi_ast::TombiValueCommentDirective]>,
    common_rules: Option<&'a tombi_comment_directive::value::CommonLintRules>,
) -> BoxFuture<'b, Option<Result<(), crate::Error>>>
where
    T: Validate + tombi_document_tree::ValueImpl + Sync + Send + std::fmt::Debug,
{
    async move {
        match (value.value_type(), resolved_schema.value_schema.as_ref()) {
            (tombi_document_tree::ValueType::Boolean, tombi_schema_store::ValueSchema::Boolean(_))
            | (
                tombi_document_tree::ValueType::Integer,
                tombi_schema_store::ValueSchema::Integer(_)
                | tombi_schema_store::ValueSchema::Float(_),
            )
            | (tombi_document_tree::ValueType::Float, tombi_schema_store::ValueSchema::Float(_))
            | (tombi_document_tree::ValueType::String, tombi_schema_store::ValueSchema::String(_))
            | (
                tombi_document_tree::ValueType::OffsetDateTime,
                tombi_schema_store::ValueSchema::OffsetDateTime(_),
            )
            | (
                tombi_document_tree::ValueType::LocalDateTime,
                tombi_schema_store::ValueSchema::LocalDateTime(_),
            )
            | (
                tombi_document_tree::ValueType::LocalDate,
                tombi_schema_store::ValueSchema::LocalDate(_),
            )
            | (
                tombi_document_tree::ValueType::LocalTime,
                tombi_schema_store::ValueSchema::LocalTime(_),
            )
            | (tombi_document_tree::ValueType::Table, tombi_schema_store::ValueSchema::Table(_))
            | (tombi_document_tree::ValueType::Array, tombi_schema_store::ValueSchema::Array(_)) => {
                Some(
                    value
                        .validate(accessors, Some(resolved_schema), schema_context)
                        .await,
                )
            }
            (_, tombi_schema_store::ValueSchema::Null) => None,
            (_, tombi_schema_store::ValueSchema::Anything(_)) => Some(handle_anything_schema(value)),
            (_, tombi_schema_store::ValueSchema::Nothing(_)) => Some(handle_nothing_schema(value)),
            (_, tombi_schema_store::ValueSchema::Boolean(_))
            | (_, tombi_schema_store::ValueSchema::Integer(_))
            | (_, tombi_schema_store::ValueSchema::Float(_))
            | (_, tombi_schema_store::ValueSchema::String(_))
            | (_, tombi_schema_store::ValueSchema::OffsetDateTime(_))
            | (_, tombi_schema_store::ValueSchema::LocalDateTime(_))
            | (_, tombi_schema_store::ValueSchema::LocalDate(_))
            | (_, tombi_schema_store::ValueSchema::LocalTime(_))
            | (_, tombi_schema_store::ValueSchema::Table(_))
            | (_, tombi_schema_store::ValueSchema::Array(_)) => Some(
                handle_type_mismatch(
                    resolved_schema.value_schema.value_type().await,
                    value.value_type(),
                    value.range(),
                    common_rules,
                ),
            ),
            (_, tombi_schema_store::ValueSchema::OneOf(one_of_schema)) => Some(
                validate_one_of(
                    value,
                    accessors,
                    one_of_schema,
                    resolved_schema,
                    schema_context,
                    comment_directives,
                    common_rules,
                )
                .await,
            ),
            (_, tombi_schema_store::ValueSchema::AnyOf(any_of_schema)) => Some(
                validate_any_of(
                    value,
                    accessors,
                    any_of_schema,
                    resolved_schema,
                    schema_context,
                    comment_directives,
                    common_rules,
                )
                .await,
            ),
            (_, tombi_schema_store::ValueSchema::AllOf(all_of_schema)) => Some(
                validate_all_of(
                    value,
                    accessors,
                    all_of_schema,
                    resolved_schema,
                    schema_context,
                    comment_directives,
                    common_rules,
                )
                .await,
            ),
        }
    }
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::{
        filter_table_strict_additional_diagnostics, is_assertion_success,
        is_multiple_of_with_tolerance,
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn assertion_success_allows_warning_only_errors() {
        let warning_only_error = crate::Error {
            score: 0,
            diagnostics: vec![tombi_diagnostic::Diagnostic::new_warning(
                "warn",
                "warn-code",
                tombi_text::Range::default(),
            )],
        };
        assert!(is_assertion_success(&Ok(())));
        assert!(is_assertion_success(&Err(warning_only_error)));
    }

    #[test]
    fn multiple_of_tolerance_handles_common_fp_noise() {
        assert!(is_multiple_of_with_tolerance(0.3, 0.1));
        assert!(is_multiple_of_with_tolerance(1.2, 0.3));
        assert!(!is_multiple_of_with_tolerance(0.31, 0.1));
    }

    #[test]
    fn filter_drops_only_table_strict_additional_diagnostics() {
        let result = filter_table_strict_additional_diagnostics(crate::Error {
            score: 1,
            diagnostics: vec![
                tombi_diagnostic::Diagnostic::new_warning(
                    "strict additional",
                    "table-strict-additional-keys",
                    tombi_text::Range::default(),
                ),
                tombi_diagnostic::Diagnostic::new_warning(
                    "other warning",
                    "deprecated",
                    tombi_text::Range::default(),
                ),
            ],
        });

        let err = result.expect_err("non-strict diagnostics should remain");
        assert_eq!(err.diagnostics.len(), 1);
        assert_eq!(err.diagnostics[0].code(), "deprecated");
    }

    #[test]
    fn filter_turns_strict_additional_only_error_into_success() {
        let result = filter_table_strict_additional_diagnostics(crate::Error {
            score: 1,
            diagnostics: vec![tombi_diagnostic::Diagnostic::new_warning(
                "strict additional",
                "table-strict-additional-keys",
                tombi_text::Range::default(),
            )],
        });

        assert!(result.is_ok());
    }
}
