use itertools::Itertools;
use tombi_comment_directive::value::{FloatCommonFormatRules, FloatCommonLintRules};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::SchemaView;
use tombi_severity_level::SeverityLevelDefaultError;

use crate::{
    comment_directive::get_tombi_key_table_value_rules_and_diagnostics,
    validate::{
        check_exclusive_maximum, check_exclusive_minimum, check_maximum, check_minimum,
        handle_anything_schema, handle_deprecated_value, handle_nothing_schema, handle_unused_noqa,
        is_multiple_of_with_tolerance, validate_adjacent_applicators,
    },
};

use super::{Validate, validate_all_of, validate_any_of, validate_one_of};

impl Validate for tombi_document_tree_syntax::Float {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<crate::Valid, crate::Invalid>> {
        async move {
            if let Some(projected_schema) = crate::validate::project_current_schema_for_value(
                self,
                current_schema,
                schema_context,
            ) {
                return self
                    .validate(accessors, Some(&projected_schema), schema_context)
                    .await;
            }

            let (lint_rules, lint_rules_diagnostics) =
                get_tombi_key_table_value_rules_and_diagnostics::<
                    FloatCommonFormatRules,
                    FloatCommonLintRules,
                >(self.comment_directives(), accessors)
                .await;

            let result = if let Some(current_schema) = current_schema {
                match current_schema.schema_view.as_ref() {
                    SchemaView::Float(float_schema) => {
                        validate_float(
                            self,
                            accessors,
                            float_schema,
                            current_schema,
                            schema_context,
                            self.comment_directives()
                                .map(|directives| directives.cloned().collect_vec())
                                .as_deref(),
                            lint_rules.as_ref(),
                        )
                        .await
                    }
                    SchemaView::OneOf(one_of_schema) => {
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
                    SchemaView::AnyOf(any_of_schema) => {
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
                    SchemaView::AllOf(all_of_schema) => {
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
                    SchemaView::Null => handle_nothing_schema(self),
                    SchemaView::Anything(_) => handle_anything_schema(self),
                    SchemaView::Nothing(_) => handle_nothing_schema(self),
                    _ => {
                        crate::validate::validate_mismatched_schema(
                            self,
                            accessors,
                            current_schema,
                            schema_context,
                            self.comment_directives()
                                .map(|directives| directives.cloned().collect_vec())
                                .as_deref(),
                            lint_rules.as_ref().map(|rules| &rules.common),
                        )
                        .await
                    }
                }
            } else {
                Ok(crate::Valid::new())
            };

            crate::validate::with_lint_diagnostics(result, lint_rules_diagnostics)
        }
        .boxed()
    }
}

async fn validate_float(
    float_value: &tombi_document_tree_syntax::Float,
    accessors: &[tombi_schema_store::Accessor],
    float_schema: &tombi_schema_store::FloatSchema,
    current_schema: &tombi_schema_store::CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    comment_directives: Option<&[tombi_ast_syntax::TombiValueCommentDirective]>,
    lint_rules: Option<&FloatCommonLintRules>,
) -> Result<crate::Valid, crate::Invalid> {
    let mut diagnostics = vec![];
    let mut assertion_failed = false;
    let mut match_evidence = Box::<crate::MatchEvidence>::default();

    let value = float_value.value();
    let range = float_value.range();

    if let Some(const_value) = &float_schema.const_value {
        let matched = value == *const_value;
        match_evidence.mark_root_value_assertion(matched, true);
        if !matched {
            assertion_failed = true;
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
                    expected: const_value.to_string(),
                    actual: value.to_string(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    } else if lint_rules
        .and_then(|rules| rules.common.const_value.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            float_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "const-value",
        );
    }

    if let Some(r#enum) = &float_schema.r#enum {
        let matched = r#enum.contains(&value);
        match_evidence.mark_root_value_assertion(matched, r#enum.len() == 1);
        if !matched {
            assertion_failed = true;
            let level = lint_rules
                .map(|rules| &rules.common)
                .and_then(|rules| rules.r#enum().map(SeverityLevelDefaultError::from))
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::Enum {
                    expected: r#enum.iter().map(ToString::to_string).collect(),
                    actual: value.to_string(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    } else if lint_rules
        .and_then(|rules| rules.common.r#enum())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            float_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "enum",
        );
    }

    if let Some(maximum) = &float_schema.maximum
        && !check_maximum(&value, maximum)
    {
        assertion_failed = true;
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .float_maximum
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::FloatMaximum {
                maximum: *maximum,
                actual: value,
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.float_maximum.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            float_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "float-maximum",
        );
    }

    if let Some(minimum) = &float_schema.minimum
        && !check_minimum(&value, minimum)
    {
        assertion_failed = true;
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .float_minimum
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::FloatMinimum {
                minimum: *minimum,
                actual: value,
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.float_minimum.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            float_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "float-minimum",
        );
    }

    if let Some(exclusive_maximum) = &float_schema.exclusive_maximum
        && !check_exclusive_maximum(&value, exclusive_maximum)
    {
        assertion_failed = true;
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .float_exclusive_maximum
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::FloatExclusiveMaximum {
                exclusive_maximum: *exclusive_maximum,
                actual: value,
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.float_exclusive_maximum.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            float_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "float-exclusive-maximum",
        );
    }

    if let Some(exclusive_minimum) = &float_schema.exclusive_minimum
        && !check_exclusive_minimum(&value, exclusive_minimum)
    {
        assertion_failed = true;
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .float_exclusive_minimum
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::FloatExclusiveMinimum {
                exclusive_minimum: *exclusive_minimum,
                actual: value,
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.float_exclusive_minimum.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            float_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "float-exclusive-minimum",
        );
    }

    if let Some(multiple_of) = &float_schema.multiple_of
        && !is_multiple_of_with_tolerance(value, *multiple_of)
    {
        assertion_failed = true;
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .float_multiple_of
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::FloatMultipleOf {
                multiple_of: *multiple_of,
                actual: value,
            }),
            range,
        }
        .push_diagnostic_with_level(level, &mut diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.float_multiple_of.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut diagnostics,
            float_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "float-multiple-of",
        );
    }

    if diagnostics.is_empty() {
        handle_deprecated_value(
            &mut diagnostics,
            float_schema.deprecation.as_ref(),
            accessors,
            float_value,
            Some(current_schema),
            schema_context,
            float_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
        );
    }

    let base_result = if diagnostics.is_empty() && !assertion_failed {
        let mut valid = crate::Valid::new();
        valid.match_evidence = match_evidence;
        Ok(valid)
    } else {
        Err(crate::Invalid {
            assertion_failed,
            match_evidence,
            diagnostics,
            local_evaluated_locations: Default::default(),
        })
    };

    crate::validate::merge_validation_results(
        base_result,
        validate_adjacent_applicators(
            float_value,
            accessors,
            float_schema.one_of.as_deref(),
            float_schema.any_of.as_deref(),
            float_schema.all_of.as_deref(),
            float_schema.not.as_deref(),
            current_schema,
            schema_context,
            comment_directives,
            lint_rules.map(|rules| &rules.common),
        )
        .await,
    )
}
