use std::borrow::Cow;

use itertools::Itertools;
use tombi_comment_directive::value::{
    CommonRules, KeyCommonExtensibleRules, KeyTableCommonRules, TableRules, WithCommonRules,
};
use tombi_document_tree::{TableKind, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, CurrentSchema, DocumentSchema, PropertySchema, SchemaAccessor, SchemaAccessors,
    ValueSchema,
};
use tombi_severity_level::{SeverityLevel, SeverityLevelDefaultError, SeverityLevelDefaultWarn};

use crate::{comment_directive::get_tombi_value_rules_and_diagnostics, validate::type_mismatch};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};
use crate::diagnostic::Patterns;

impl Validate for tombi_document_tree::Table {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut total_diagnostics = vec![];
            let (value_rules, common_rules) = if let Some(comment_directives) =
                self.comment_directives()
            {
                let (value_rules, common_rules, diagnostics) = if self.kind() == TableKind::KeyValue
                    && self
                        .values()
                        .next()
                        .map(|value| value.is_inline())
                        .unwrap_or_default()
                {
                    let (rules, diagnostics) = get_tombi_value_rules_and_diagnostics::<
                        KeyCommonExtensibleRules,
                    >(comment_directives)
                    .await;

                    if let Some(KeyCommonExtensibleRules { common, .. }) = rules {
                        (None, Some(common), diagnostics)
                    } else {
                        (None, None, diagnostics)
                    }
                } else {
                    let (rules, diagnostics) = get_tombi_value_rules_and_diagnostics::<
                        KeyTableCommonRules,
                    >(comment_directives)
                    .await;

                    if let Some(KeyTableCommonRules {
                        value:
                            WithCommonRules {
                                common,
                                value: table,
                            },
                        ..
                    }) = rules
                    {
                        (Some(table), Some(common), diagnostics)
                    } else {
                        (None, None, diagnostics)
                    }
                };

                total_diagnostics.extend(diagnostics);

                (value_rules, common_rules)
            } else {
                (None, None)
            };

            if let Some(sub_schema_uri) = schema_context
                .sub_schema_uri_map
                .and_then(|map| map.get(&accessors.into_iter().map(Into::into).collect_vec()))
            {
                if current_schema.map(|schema| schema.schema_uri.as_ref()) != Some(sub_schema_uri) {
                    if let Ok(Some(DocumentSchema {
                        value_schema: Some(value_schema),
                        schema_uri,
                        definitions,
                        ..
                    })) = schema_context
                        .store
                        .try_get_document_schema(sub_schema_uri)
                        .await
                    {
                        return self
                            .validate(
                                accessors,
                                Some(&CurrentSchema {
                                    value_schema: Cow::Borrowed(&value_schema),
                                    schema_uri: Cow::Borrowed(&schema_uri),
                                    definitions: Cow::Borrowed(&definitions),
                                }),
                                schema_context,
                            )
                            .await;
                    }
                }
            }

            if let Some(current_schema) = current_schema {
                let result = match current_schema.value_schema.as_ref() {
                    ValueSchema::Table(table_schema) => {
                        validate_table(
                            self,
                            accessors,
                            table_schema,
                            current_schema,
                            schema_context,
                            value_rules.as_ref(),
                            common_rules.as_ref(),
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
                            common_rules.as_ref(),
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
                            common_rules.as_ref(),
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
                            common_rules.as_ref(),
                        )
                        .await
                    }
                    ValueSchema::Null => return Ok(()),
                    value_schema => type_mismatch(
                        value_schema.value_type().await,
                        self.value_type(),
                        self.range(),
                        common_rules.as_ref(),
                    ),
                };

                if let Err(diagnostics) = result {
                    total_diagnostics.extend(diagnostics);
                }
            } else {
                if let Err(diagnostics) =
                    validate_table_without_schema(self, accessors, schema_context).await
                {
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

async fn validate_table(
    table_value: &tombi_document_tree::Table,
    accessors: &[tombi_schema_store::Accessor],
    table_schema: &tombi_schema_store::TableSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    table_rules: Option<&TableRules>,
    common_rules: Option<&CommonRules>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];

    for (key, value) in table_value.key_values() {
        let key_rules = if let Some(directives) = key.comment_directives() {
            get_tombi_value_rules_and_diagnostics::<KeyCommonExtensibleRules>(&directives)
                .await
                .0
                .map(|rules| rules.value)
        } else {
            None
        };
        let key_rules = key_rules.as_ref();

        let accessor_raw_text = key.to_raw_text(schema_context.toml_version);
        let accessor = Accessor::Key(accessor_raw_text.clone());
        let new_accessors = accessors
            .iter()
            .cloned()
            .chain(std::iter::once(Accessor::Key(accessor_raw_text.clone())))
            .collect_vec();

        let mut matched_key = false;
        if let Some(PropertySchema {
            property_schema, ..
        }) = table_schema
            .properties
            .write()
            .await
            .get_mut(&SchemaAccessor::from(&accessor))
        {
            matched_key = true;

            if let Ok(Some(current_schema)) = property_schema
                .resolve(
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| tracing::warn!("{err}"))
            {
                if let Err(mut schema_diagnostics) = value
                    .validate(&new_accessors, Some(&current_schema), schema_context)
                    .await
                {
                    convert_deprecated_diagnostics_range(
                        &current_schema,
                        value,
                        key,
                        &mut schema_diagnostics,
                    )
                    .await;

                    diagnostics.extend(schema_diagnostics);
                }
            }
        }

        if let Some(pattern_properties) = &table_schema.pattern_properties {
            for (
                pattern_key,
                PropertySchema {
                    property_schema, ..
                },
            ) in pattern_properties.write().await.iter_mut()
            {
                let Ok(pattern) = regex::Regex::new(pattern_key) else {
                    tracing::warn!("Invalid regex pattern property: {}", pattern_key);
                    continue;
                };
                if pattern.is_match(&accessor_raw_text) {
                    matched_key = true;
                    if let Ok(Some(current_schema)) = property_schema
                        .resolve(
                            current_schema.schema_uri.clone(),
                            current_schema.definitions.clone(),
                            schema_context.store,
                        )
                        .await
                        .inspect_err(|err| tracing::warn!("{err}"))
                    {
                        if let Err(mut schema_diagnostics) = value
                            .validate(&new_accessors, Some(&current_schema), schema_context)
                            .await
                        {
                            convert_deprecated_diagnostics_range(
                                &current_schema,
                                value,
                                key,
                                &mut schema_diagnostics,
                            )
                            .await;

                            diagnostics.extend(schema_diagnostics);
                        }
                    }
                } else if !table_schema.allows_additional_properties(schema_context.strict()) {
                    let level = key_rules
                        .and_then(|rules| {
                            rules
                                .key_pattern
                                .as_ref()
                                .map(SeverityLevelDefaultError::from)
                        })
                        .unwrap_or_default();

                    crate::Diagnostic {
                        kind: Box::new(crate::DiagnosticKind::KeyPattern {
                            patterns: Patterns(
                                pattern_properties
                                    .read()
                                    .await
                                    .keys()
                                    .map(ToString::to_string)
                                    .collect(),
                            ),
                        }),
                        range: key.range(),
                    }
                    .push_diagnostic_with_level(level, &mut diagnostics);
                }
            }
        }
        if !matched_key {
            if let Some((_, referable_additional_property_schema)) =
                &table_schema.additional_property_schema
            {
                let mut referable_schema = referable_additional_property_schema.write().await;
                if let Ok(Some(current_schema)) = referable_schema
                    .resolve(
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                    )
                    .await
                    .inspect_err(|err| tracing::warn!("{err}"))
                {
                    if current_schema.value_schema.deprecated().await == Some(true) {
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
                                SchemaAccessors::from(&new_accessors),
                                value.to_string(),
                            )),
                            range: key.range() + value.range(),
                        }
                        .push_diagnostic_with_level(level, &mut diagnostics);
                    }

                    if let Err(schema_diagnostics) = value
                        .validate(&new_accessors, Some(&current_schema), schema_context)
                        .await
                    {
                        diagnostics.extend(schema_diagnostics);
                    }
                }
            }
            if table_schema.check_strict_additional_properties_violation(schema_context.strict()) {
                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::StrictAdditionalProperties {
                        accessors: SchemaAccessors::from(accessors),
                        schema_uri: current_schema.schema_uri.as_ref().clone(),
                        key: key.to_string(),
                    }),
                    range: key.range() + value.range(),
                }
                .push_diagnostic_with_level(SeverityLevel::Warn, &mut diagnostics);

                continue;
            }
            if !table_schema.allows_any_additional_properties(schema_context.strict()) {
                let level = key_rules
                    .and_then(|rules| {
                        rules
                            .key_not_allowed
                            .as_ref()
                            .map(SeverityLevelDefaultError::from)
                    })
                    .unwrap_or_default();

                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::KeyNotAllowed {
                        key: key.to_string(),
                    }),
                    range: key.range() + value.range(),
                }
                .push_diagnostic_with_level(level, &mut diagnostics);
                continue;
            }
        }
    }

    if let Some(required) = &table_schema.required {
        let keys = table_value
            .keys()
            .map(|key| key.to_raw_text(schema_context.toml_version))
            .collect_vec();

        for required_key in required {
            if !keys.contains(required_key) {
                let level = table_rules
                    .and_then(|rules| {
                        rules
                            .key_required
                            .as_ref()
                            .map(SeverityLevelDefaultError::from)
                    })
                    .unwrap_or_default();

                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::KeyRequired {
                        key: required_key.to_string(),
                    }),
                    range: table_value.range(),
                }
                .push_diagnostic_with_level(level, &mut diagnostics);
            }
        }
    }

    if let Some(max_properties) = table_schema.max_properties {
        if table_value.keys().count() > max_properties {
            let level = table_rules
                .and_then(|rules| {
                    rules
                        .table_max_keys
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::TableMaxKeys {
                    max_keys: max_properties,
                    actual: table_value.keys().count(),
                }),
                range: table_value.range(),
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(min_properties) = table_schema.min_properties {
        if table_value.keys().count() < min_properties {
            let level = table_rules
                .and_then(|rules| {
                    rules
                        .table_min_keys
                        .as_ref()
                        .map(SeverityLevelDefaultError::from)
                })
                .unwrap_or_default();

            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::TableMinKeys {
                    min_keys: min_properties,
                    actual: table_value.keys().count(),
                }),
                range: table_value.range(),
            }
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        if table_schema.deprecated == Some(true) {
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
                range: table_value.range(),
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

async fn validate_table_without_schema(
    table_value: &tombi_document_tree::Table,
    accessors: &[tombi_schema_store::Accessor],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];

    // Validate without schema
    for (key, value) in table_value.key_values() {
        if let Err(schema_diagnostics) = value
            .validate(
                &accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(Accessor::Key(
                        key.to_raw_text(schema_context.toml_version),
                    )))
                    .collect_vec(),
                None,
                schema_context,
            )
            .await
        {
            diagnostics.extend(schema_diagnostics);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Convert deprecated diagnostics to warnings for the given value
async fn convert_deprecated_diagnostics_range(
    current_schema: &CurrentSchema<'_>,
    value: &tombi_document_tree::Value,
    key: &tombi_document_tree::Key,
    schema_diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if current_schema.value_schema.deprecated().await == Some(true) {
        for diagnostic in schema_diagnostics.iter_mut() {
            if diagnostic.code() == "deprecated" && diagnostic.range() == value.range() {
                *diagnostic = tombi_diagnostic::Diagnostic::new_warning(
                    diagnostic.message(),
                    diagnostic.code(),
                    key.range() + value.range(),
                );
                break;
            }
        }
    }
}
