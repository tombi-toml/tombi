use tombi_comment_directive::value::OffsetDateTimeCommonRules;
use tombi_document_tree::{OffsetDateTime, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;
use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};

use crate::{
    comment_directive::get_tombi_key_table_value_rules_and_diagnostics,
    validate::type_mismatch,
};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for OffsetDateTime {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut total_diagnostics = vec![];
            let value_rules = if let Some(comment_directives) = self.comment_directives() {
                let (value_rules, diagnostics) = get_tombi_key_table_value_rules_and_diagnostics::<
                    OffsetDateTimeCommonRules,
                >(comment_directives, accessors)
                .await;

                total_diagnostics.extend(diagnostics);

                value_rules
            } else {
                None
            };

            if let Some(current_schema) = current_schema {
                let result = match current_schema.value_schema.as_ref() {
                    ValueSchema::OffsetDateTime(offset_date_time_schema) => {
                        validate_offset_date_time(
                            self,
                            accessors,
                            offset_date_time_schema,
                            value_rules.as_ref(),
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
                            value_rules.as_ref().map(|rules| &rules.common),
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
                            value_rules.as_ref().map(|rules| &rules.common),
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

async fn validate_offset_date_time(
    offset_date_time_value: &OffsetDateTime,
    accessors: &[tombi_schema_store::Accessor],
    offset_date_time_schema: &tombi_schema_store::OffsetDateTimeSchema,
    value_rules: Option<&OffsetDateTimeCommonRules>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    let value_string = offset_date_time_value.value().to_string();
    let range = offset_date_time_value.range();

    if let Some(const_value) = &offset_date_time_schema.const_value {
        if value_string != *const_value {
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
                    expected: const_value.clone(),
                    actual: value_string.clone(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(enumerate) = &offset_date_time_schema.enumerate {
        if !enumerate.contains(&value_string) {
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
                    actual: value_string.clone(),
                }),
                range,
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        if offset_date_time_schema.deprecated == Some(true) {
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
                    value_string,
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
