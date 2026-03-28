use itertools::Itertools;
use tombi_comment_directive::value::{
    LocalDateTimeCommonFormatRules, LocalDateTimeCommonLintRules,
};
use tombi_document_tree::{LocalDateTime, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;
use tombi_severity_level::SeverityLevelDefaultError;

use crate::{
    comment_directive::get_tombi_key_table_value_rules_and_diagnostics,
    validate::{
        handle_anything_schema, handle_deprecated_value, handle_nothing_schema,
        handle_type_mismatch, handle_unused_noqa, validate_adjacent_applicators,
    },
};

use super::{Validate, validate_all_of, validate_any_of, validate_one_of};

impl Validate for LocalDateTime {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<crate::EvaluatedLocations, crate::Error>> {
        async move {
            let (lint_rules, lint_rules_diagnostics) =
                get_tombi_key_table_value_rules_and_diagnostics::<
                    LocalDateTimeCommonFormatRules,
                    LocalDateTimeCommonLintRules,
                >(self.comment_directives(), accessors)
                .await;

            let result = if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::LocalDateTime(local_date_time_schema) => {
                        validate_local_date_time(
                            self,
                            accessors,
                            local_date_time_schema,
                            current_schema,
                            schema_context,
                            self.comment_directives()
                                .map(|directives| directives.cloned().collect_vec())
                                .as_deref(),
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
                    ValueSchema::Null => return Ok(crate::EvaluatedLocations::new()),
                    ValueSchema::Anything(_) => handle_anything_schema(self),
                    ValueSchema::Nothing(_) => handle_nothing_schema(self),
                    value_schema => handle_type_mismatch(
                        value_schema.value_type().await,
                        self.value_type(),
                        self.range(),
                        lint_rules.as_ref().map(|rules| &rules.common),
                    ),
                }
            } else {
                Ok(crate::EvaluatedLocations::new())
            };

            crate::validate::with_lint_diagnostics(result, lint_rules_diagnostics)
        }
        .boxed()
    }
}

async fn validate_local_date_time(
    local_date_time_value: &LocalDateTime,
    accessors: &[tombi_schema_store::Accessor],
    local_date_time_schema: &tombi_schema_store::LocalDateTimeSchema,
    current_schema: &tombi_schema_store::CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    comment_directives: Option<&[tombi_ast::TombiValueCommentDirective]>,
    lint_rules: Option<&LocalDateTimeCommonLintRules>,
) -> Result<crate::EvaluatedLocations, crate::Error> {
    let mut diagnostics = vec![];
    let value_string = local_date_time_value.value().to_string();
    let range = local_date_time_value.range();

    if let Some(const_value) = &local_date_time_schema.const_value
        && value_string != *const_value
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
                expected: const_value.clone(),
                actual: value_string.clone(),
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
            local_date_time_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "const-value",
        );
    }

    if let Some(r#enum) = &local_date_time_schema.r#enum
        && !r#enum.contains(&value_string)
    {
        let level = lint_rules
            .map(|rules| &rules.common)
            .and_then(|rules| rules.r#enum().map(SeverityLevelDefaultError::from))
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::Enum {
                expected: r#enum.iter().map(ToString::to_string).collect(),
                actual: value_string.clone(),
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
            local_date_time_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "enum",
        );
    }

    if diagnostics.is_empty() {
        handle_deprecated_value(
            &mut diagnostics,
            local_date_time_schema.deprecated,
            accessors,
            local_date_time_value,
            Some(current_schema),
            schema_context,
            local_date_time_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
        );
    }

    let base_result = if diagnostics.is_empty() {
        Ok(crate::EvaluatedLocations::new())
    } else {
        Err(diagnostics.into())
    };

    crate::validate::merge_validation_results(
        base_result,
        validate_adjacent_applicators(
            local_date_time_value,
            accessors,
            local_date_time_schema.one_of.as_deref(),
            local_date_time_schema.any_of.as_deref(),
            local_date_time_schema.all_of.as_deref(),
            local_date_time_schema.not.as_deref(),
            current_schema,
            schema_context,
            comment_directives,
            lint_rules.map(|rules| &rules.common),
        )
        .await,
    )
}
