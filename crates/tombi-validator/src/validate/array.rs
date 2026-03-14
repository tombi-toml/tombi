use std::borrow::Cow;

use itertools::Itertools;
use tombi_comment_directive::value::ArrayCommonLintRules;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, ValueSchema};
use tombi_severity_level::SeverityLevelDefaultError;

use crate::{
    comment_directive::get_tombi_array_comment_directive_and_diagnostics,
    validate::{
        handle_anything_schema, handle_deprecated, handle_nothing_schema, handle_type_mismatch,
        handle_unused_noqa, if_then_else::validate_if_then_else, is_assertion_success,
        merge_validation_results, validate_adjacent_applicators,
    },
};

use super::{Validate, validate_all_of, validate_any_of, validate_one_of};

impl Validate for tombi_document_tree::Array {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), crate::Error>> {
        let comment_directives = self
            .comment_directives()
            .map(|directives| directives.cloned().collect_vec());

        async move {
            if let Some(Ok(document_schema)) = schema_context
                .get_subschema(accessors, current_schema)
                .await
                && let Some(value_schema) = &document_schema.value_schema
            {
                return self
                    .validate(
                        accessors,
                        Some(&CurrentSchema {
                            value_schema: value_schema.clone(),
                            schema_uri: Cow::Borrowed(&document_schema.schema_uri),
                            definitions: Cow::Borrowed(&document_schema.definitions),
                        }),
                        schema_context,
                    )
                    .await;
            }

            let (lint_rules, lint_rules_diagnostics) =
                get_tombi_array_comment_directive_and_diagnostics(self, accessors).await;

            let result = if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Array(array_schema) => {
                        validate_array(
                            self,
                            accessors,
                            array_schema,
                            current_schema,
                            schema_context,
                            comment_directives.as_deref(),
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
                            comment_directives.as_deref(),
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
                            comment_directives.as_deref(),
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
                            comment_directives.as_deref(),
                            lint_rules.as_ref().map(|rules| &rules.common),
                        )
                        .await
                    }
                    ValueSchema::Null => return Ok(()),
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
                validate_array_without_schema(self, accessors, schema_context).await
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

async fn validate_array(
    array_value: &tombi_document_tree::Array,
    accessors: &[tombi_schema_store::Accessor],
    array_schema: &tombi_schema_store::ArraySchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    comment_directives: Option<&[tombi_ast::TombiValueCommentDirective]>,
    lint_rules: Option<&ArrayCommonLintRules>,
) -> Result<(), crate::Error> {
    let mut total_diagnostics = vec![];
    let mut evaluated = vec![false; array_value.values().len()];
    let has_unevaluated_items = array_schema.unevaluated_items_schema.is_some()
        || array_schema.unevaluated_items == Some(false);

    if let Some(if_then_else_schema) = array_schema.if_then_else.as_ref()
        && let Err(error) = validate_if_then_else(
            array_value,
            accessors,
            if_then_else_schema,
            current_schema,
            schema_context,
        )
        .await
    {
        total_diagnostics.extend(error.diagnostics);
    }

    if let Some(prefix_items) = &array_schema.prefix_items {
        // Resolve the overflow schema once before the loop
        let overflow_schema =
            if let Some(additional_items_schema) = &array_schema.additional_items_schema {
                tombi_schema_store::resolve_schema_item(
                    additional_items_schema,
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| log::warn!("{err}"))
                .ok()
                .flatten()
            } else if let Some(items) = &array_schema.items {
                // 2020-12: items acts as additionalItems when prefixItems is present
                tombi_schema_store::resolve_schema_item(
                    items,
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| log::warn!("{err}"))
                .ok()
                .flatten()
            } else {
                None
            };

        // Tuple validation: validate each element against its positional schema
        for (index, value) in array_value.values().iter().enumerate() {
            let new_accessors = accessors
                .iter()
                .cloned()
                .chain(std::iter::once(tombi_schema_store::Accessor::Index(index)))
                .collect_vec();

            if index < prefix_items.len() {
                evaluated[index] = true;
                if let Ok(Some(item_schema)) = tombi_schema_store::resolve_schema_item(
                    &prefix_items[index],
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| log::warn!("{err}"))
                    && let Err(crate::Error { diagnostics, .. }) = value
                        .validate(&new_accessors, Some(&item_schema), schema_context)
                        .await
                {
                    total_diagnostics.extend(diagnostics);
                }
            } else if let Some(overflow) = &overflow_schema {
                evaluated[index] = true;
                if let Err(crate::Error { diagnostics, .. }) = value
                    .validate(&new_accessors, Some(overflow), schema_context)
                    .await
                {
                    total_diagnostics.extend(diagnostics);
                }
            } else if array_schema.additional_items == Some(false) {
                if has_unevaluated_items {
                    evaluated[index] = true;
                }
                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::ArrayAdditionalItems {
                        max_items: prefix_items.len(),
                    }),
                    range: value.range(),
                }
                .push_diagnostic_with_level(
                    SeverityLevelDefaultError::default(),
                    &mut total_diagnostics,
                );
            }
        }
    } else if let Some(items) = &array_schema.items {
        // Single schema for all items
        if let Ok(Some(current_schema)) = tombi_schema_store::resolve_schema_item(
            items,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
        )
        .await
        .inspect_err(|err| log::warn!("{err}"))
        {
            for (index, value) in array_value.values().iter().enumerate() {
                evaluated[index] = true;
                let new_accessors = accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(tombi_schema_store::Accessor::Index(index)))
                    .collect_vec();

                if let Err(crate::Error { diagnostics, .. }) = value
                    .validate(&new_accessors, Some(&current_schema), schema_context)
                    .await
                {
                    total_diagnostics.extend(diagnostics);
                }
            }
        }
    }

    if let Some(contains) = &array_schema.contains
        && let Ok(Some(contains_schema)) = tombi_schema_store::resolve_schema_item(
            contains,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
        )
        .await
        .inspect_err(|err| log::warn!("{err}"))
    {
        let min_contains = array_schema.min_contains.unwrap_or(1);
        let max_contains = array_schema.max_contains;
        let needs_full_count = max_contains.is_some() || has_unevaluated_items;
        let mut contains_evaluated = vec![false; array_value.values().len()];

        let mut match_count = 0usize;
        for (index, value) in array_value.values().iter().enumerate() {
            let new_accessors = accessors
                .iter()
                .cloned()
                .chain(std::iter::once(tombi_schema_store::Accessor::Index(index)))
                .collect_vec();

            let result = value
                .validate(&new_accessors, Some(&contains_schema), schema_context)
                .await;
            if is_assertion_success(&result) {
                contains_evaluated[index] = true;
                match_count += 1;
                if !needs_full_count && match_count >= min_contains {
                    break;
                }
            }
        }

        if match_count < min_contains {
            if array_schema.min_contains.is_some() {
                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::ArrayMinContains {
                        min_contains,
                        actual: match_count,
                    }),
                    range: array_value.range(),
                }
                .push_diagnostic_with_level(
                    SeverityLevelDefaultError::default(),
                    &mut total_diagnostics,
                );
            } else {
                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::ArrayContains),
                    range: array_value.range(),
                }
                .push_diagnostic_with_level(
                    SeverityLevelDefaultError::default(),
                    &mut total_diagnostics,
                );
            }
        }

        if let Some(max) = max_contains
            && match_count > max
        {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::ArrayMaxContains {
                    max_contains: max,
                    actual: match_count,
                }),
                range: array_value.range(),
            }
            .push_diagnostic_with_level(
                SeverityLevelDefaultError::default(),
                &mut total_diagnostics,
            );
        }

        for (index, matched) in contains_evaluated.iter().enumerate() {
            if *matched {
                evaluated[index] = true;
            }
        }
    }

    // Run unevaluatedItems after all applicators that can mark items as evaluated.
    if has_unevaluated_items {
        let unevaluated_schema = if let Some(schema_item) = &array_schema.unevaluated_items_schema {
            tombi_schema_store::resolve_schema_item(
                schema_item,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
            .inspect_err(|err| log::warn!("{err}"))
            .ok()
            .flatten()
        } else {
            None
        };

        for (index, value) in array_value.values().iter().enumerate() {
            if evaluated.get(index).copied().unwrap_or(false) {
                continue;
            }
            if let Some(schema) = &unevaluated_schema {
                let new_accessors = accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(tombi_schema_store::Accessor::Index(index)))
                    .collect_vec();
                if let Err(crate::Error { diagnostics, .. }) = value
                    .validate(&new_accessors, Some(schema), schema_context)
                    .await
                {
                    total_diagnostics.extend(diagnostics);
                }
            } else if array_schema.unevaluated_items == Some(false) {
                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::ArrayUnevaluatedItemNotAllowed { index }),
                    range: value.range(),
                }
                .push_diagnostic_with_level(
                    SeverityLevelDefaultError::default(),
                    &mut total_diagnostics,
                );
            }
        }
    }

    if array_schema.const_value.is_some() || array_schema.r#enum.is_some() {
        let actual_value = tombi_json_value::Value::Array(
            array_value
                .values()
                .iter()
                .map(crate::convert::value_to_json_value)
                .collect(),
        );

        if let Some(const_value) = &array_schema.const_value {
            if actual_value != *const_value {
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
                        actual: actual_value.to_string(),
                    }),
                    range: array_value.range(),
                }
                .push_diagnostic_with_level(level, &mut total_diagnostics);
            }
        } else if lint_rules
            .and_then(|rules| rules.common.const_value.as_ref())
            .and_then(|rules| rules.disabled)
            == Some(true)
        {
            handle_unused_noqa(
                &mut total_diagnostics,
                array_value.comment_directives(),
                lint_rules.as_ref().map(|rules| &rules.common),
                "const-value",
            );
        }

        if let Some(r#enum) = &array_schema.r#enum {
            if !r#enum.contains(&actual_value) {
                let level = lint_rules
                    .map(|rules| &rules.common)
                    .and_then(|rules| rules.r#enum().map(SeverityLevelDefaultError::from))
                    .unwrap_or_default();

                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::Enum {
                        expected: r#enum.iter().map(|item| item.to_string()).collect(),
                        actual: actual_value.to_string(),
                    }),
                    range: array_value.range(),
                }
                .push_diagnostic_with_level(level, &mut total_diagnostics);
            }
        } else if lint_rules
            .and_then(|rules| rules.common.r#enum())
            .and_then(|rules| rules.disabled)
            == Some(true)
        {
            handle_unused_noqa(
                &mut total_diagnostics,
                array_value.comment_directives(),
                lint_rules.as_ref().map(|rules| &rules.common),
                "enum",
            );
        }
    }

    if let Some(max_items) = array_schema.max_items
        && array_value.values().len() > max_items
    {
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .array_max_values
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::ArrayMaxValues {
                max_values: max_items,
                actual: array_value.values().len(),
            }),
            range: array_value.range(),
        }
        .push_diagnostic_with_level(level, &mut total_diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.array_max_values.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut total_diagnostics,
            array_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "array-max-values",
        );
    }

    if let Some(min_items) = array_schema.min_items
        && array_value.values().len() < min_items
    {
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .array_min_values
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::ArrayMinValues {
                min_values: min_items,
                actual: array_value.values().len(),
            }),
            range: array_value.range(),
        }
        .push_diagnostic_with_level(level, &mut total_diagnostics);
    } else if lint_rules
        .and_then(|rules| rules.value.array_min_values.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut total_diagnostics,
            array_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "array-min-values",
        );
    }

    if array_schema.unique_items == Some(true)
        && let Some(duplicated_ranges) = get_duplicated_ranges(array_value)
    {
        let level = lint_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .array_unique_values
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        for range in duplicated_ranges {
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::ArrayUniqueValues),
                range,
            }
            .push_diagnostic_with_level(level, &mut total_diagnostics);
        }
    } else if lint_rules
        .and_then(|rules| rules.value.array_unique_values.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut total_diagnostics,
            array_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
            "array-unique-values",
        );
    }

    if total_diagnostics.is_empty() {
        handle_deprecated(
            &mut total_diagnostics,
            array_schema.deprecated,
            accessors,
            array_value,
            array_value.comment_directives(),
            lint_rules.as_ref().map(|rules| &rules.common),
        );
    }

    let base_result = if total_diagnostics.is_empty() {
        Ok(())
    } else {
        Err(total_diagnostics.into())
    };

    merge_validation_results(
        base_result,
        validate_adjacent_applicators(
            array_value,
            accessors,
            array_schema.one_of.as_deref(),
            array_schema.any_of.as_deref(),
            array_schema.all_of.as_deref(),
            array_schema.not.as_deref(),
            current_schema,
            schema_context,
            comment_directives,
            lint_rules.map(|rules| &rules.common),
        )
        .await,
    )
}

async fn validate_array_without_schema(
    array_value: &tombi_document_tree::Array,
    accessors: &[tombi_schema_store::Accessor],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Result<(), crate::Error> {
    let mut total_diagnostics = vec![];

    // Validate without schema
    for (index, value) in array_value.values().iter().enumerate() {
        if let Err(crate::Error { diagnostics, .. }) = value
            .validate(
                &accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(tombi_schema_store::Accessor::Index(index)))
                    .collect_vec(),
                None,
                schema_context,
            )
            .await
        {
            total_diagnostics.extend(diagnostics);
        }
    }

    if total_diagnostics.is_empty() {
        Ok(())
    } else {
        Err(total_diagnostics.into())
    }
}

fn get_duplicated_ranges(
    array_value: &tombi_document_tree::Array,
) -> Option<Vec<tombi_text::Range>> {
    let values = array_value
        .values()
        .iter()
        .map(crate::convert::value_to_json_value)
        .collect_vec();

    let duplicated_ranges = array_value
        .values()
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let current = &values[index];
            let is_duplicated = values
                .iter()
                .enumerate()
                .any(|(other_index, other)| other_index != index && other == current);
            is_duplicated.then_some(value.range())
        })
        .collect_vec();

    (!duplicated_ranges.is_empty()).then_some(duplicated_ranges)
}
