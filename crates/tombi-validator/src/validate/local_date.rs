use tombi_comment_directive::value::{LocalDateCommonFormatRules, LocalDateCommonLintRules};
use tombi_document_tree::{LocalDate, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;
use tombi_severity_level::SeverityLevelDefaultError;

use crate::{
    comment_directive::get_tombi_key_table_value_rules_and_diagnostics,
    validate::{push_deprecated_value, type_mismatch},
};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for LocalDate {
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
                        LocalDateCommonFormatRules,
                        LocalDateCommonLintRules,
                    >(comment_directives, accessors)
                    .await
                } else {
                    (None, Vec::with_capacity(0))
                };

            let result = if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::LocalDate(local_date_schema) => {
                        validate_local_date(self, accessors, local_date_schema, lint_rules.as_ref())
                            .await
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

async fn validate_local_date(
    local_date_value: &LocalDate,
    accessors: &[tombi_schema_store::Accessor],
    local_date_schema: &tombi_schema_store::LocalDateSchema,
    lint_rules: Option<&LocalDateCommonLintRules>,
) -> Result<(), crate::Error> {
    let mut diagnostics = vec![];
    let value_string = local_date_value.value().to_string();
    let range = local_date_value.range();

    if let Some(const_value) = &local_date_schema.const_value {
        if value_string != *const_value {
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
        }
    }

    if let Some(enumerate) = &local_date_schema.enumerate {
        if !enumerate.contains(&value_string) {
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
                    expected: enumerate.iter().map(ToString::to_string).collect(),
                    actual: value_string.clone(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() && local_date_schema.deprecated == Some(true) {
        push_deprecated_value(
            &mut diagnostics,
            accessors,
            local_date_value,
            lint_rules.as_ref().map(|rules| &rules.common),
        );
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.into())
    }
}
