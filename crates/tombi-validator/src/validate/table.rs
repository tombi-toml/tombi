use itertools::Itertools;
use tombi_accessor::MarkdownSchemaAccessors;
use tombi_comment_directive::value::TableCommonLintRules;
use tombi_future::{BoxFuture, Boxable};
use tombi_hashmap::HashMap;
use tombi_schema_store::{Accessor, CompositeSchema, CurrentSchema, SchemaAccessor, SchemaView};
use tombi_severity_level::{SeverityLevel, SeverityLevelDefaultError};

use crate::{
    comment_directive::{
        get_tombi_key_rules_and_diagnostics, get_tombi_table_comment_directive_and_diagnostics,
    },
    validate::{
        filter_table_strict_additional_diagnostics, handle_anything_schema, handle_deprecated,
        handle_deprecated_value, handle_nothing_schema, handle_unused_noqa,
        if_then_else::validate_if_then_else, merge_validation_results,
        validate_adjacent_applicators,
    },
};

use super::{Validate, validate_all_of, validate_any_of, validate_one_of};
use crate::diagnostic::Patterns;

/// A dependency schema is an additional constraint layered on top of the parent
/// table schema. Running strict mode here against the partial dependency schema
/// causes false-positive additional key diagnostics for valid keys defined by
/// the parent schema.
fn dependency_schema_context<'a>(
    schema_context: &tombi_schema_store::SchemaContext<'a>,
) -> tombi_schema_store::SchemaContext<'a> {
    tombi_schema_store::SchemaContext {
        toml_version: schema_context.toml_version,
        root_schema: schema_context.root_schema,
        sub_schema_link_map: schema_context.sub_schema_link_map,
        deprecated_lint_level: schema_context.deprecated_lint_level,
        schema_format_rules: schema_context.schema_format_rules,
        schema_lint_rules: schema_context.schema_lint_rules,
        schema_overrides: schema_context.schema_overrides,
        schema_visits: schema_context.schema_visits.clone(),
        store: schema_context.store,
        strict: Some(false.into()),
    }
}

/// Validates every dependent schema whose trigger key is present, merging the
/// annotations it produced into `evaluated_locations`.
///
/// `unevaluatedProperties` is decided while walking the keys, so these have to
/// run before that walk; the failures they leave behind are returned so the
/// caller can report them where the owning keyword is diagnosed, keeping the
/// diagnostic order the keyword would have produced on its own. Only the
/// diagnostics and `assertion_failed` of a returned failure are reported; its
/// `match_evidence` does not propagate out of a dependent schema.
async fn validate_dependent_schemas<'a>(
    entries: &[(&'a str, &'a tombi_schema_store::SchemaItem)],
    table_value: &tombi_document_tree_syntax::Table,
    accessors: &[tombi_schema_store::Accessor],
    keys: &[&str],
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
    evaluated_locations: &mut crate::Valid,
) -> HashMap<&'a str, crate::Invalid> {
    let dependency_context = dependency_schema_context(schema_context);
    let mut failures = HashMap::new();

    for (dependent_key, schema_item) in entries {
        if !keys.contains(dependent_key) {
            continue;
        }

        match tombi_schema_store::resolve_schema_item(
            schema_item,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            current_schema.strict,
            schema_context.store,
        )
        .await
        {
            Ok(Some(dependent_schema)) => {
                let result = table_value
                    .validate(accessors, Some(&dependent_schema), &dependency_context)
                    .await;

                if let Some(error) = evaluated_locations.merge_result(result) {
                    failures.insert(*dependent_key, error);
                }
            }
            Ok(None) => {}
            Err(err) => {
                if let Some(diagnostic) = crate::validate::schema_resolution_diagnostic(
                    &err,
                    table_value.range(),
                    common_rules,
                ) {
                    failures.insert(
                        *dependent_key,
                        crate::Invalid {
                            diagnostics: vec![diagnostic],
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    failures
}

impl Validate for tombi_document_tree_syntax::Table {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<crate::Valid, crate::Invalid>> {
        async move {
            if let Some(Ok(current_schema)) = schema_context
                .get_subschema(accessors, current_schema)
                .await
            {
                return self
                    .validate(accessors, Some(&current_schema), schema_context)
                    .await;
            }

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
                get_tombi_table_comment_directive_and_diagnostics(self, accessors).await;

            let result = if let Some(current_schema) = current_schema {
                match current_schema.schema_view.as_ref() {
                    SchemaView::Table(table_schema) => {
                        validate_table(
                            self,
                            accessors,
                            table_schema,
                            current_schema,
                            schema_context,
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
                validate_table_without_schema(self, accessors, schema_context).await
            };

            crate::validate::with_lint_diagnostics(result, lint_rules_diagnostics)
        }
        .boxed()
    }
}

async fn validate_table(
    table_value: &tombi_document_tree_syntax::Table,
    accessors: &[tombi_schema_store::Accessor],
    table_schema: &tombi_schema_store::TableSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    table_rules: Option<&TableCommonLintRules>,
) -> Result<crate::Valid, crate::Invalid> {
    let mut match_evidence = Box::<crate::MatchEvidence>::default();
    let mut assertion_failed = false;
    let mut total_diagnostics = vec![];
    let common_rules = table_rules.map(|rules| &rules.common);
    let keys = table_value.keys().map(|key| key.value()).collect_vec();
    let mut evaluated_locations = collect_evaluated_properties_from_table_schema(
        table_value,
        accessors,
        table_schema,
        current_schema,
        schema_context,
    )
    .await;

    let mut dependency_schema_failures = HashMap::new();
    if let Some(dependencies) = &table_schema.dependencies {
        let entries = dependencies
            .iter()
            .filter_map(|(key, dependency)| match dependency {
                tombi_schema_store::Dependency::Schema(schema_item) => {
                    Some((key.as_str(), schema_item))
                }
                tombi_schema_store::Dependency::Property(_) => None,
            })
            .collect_vec();
        dependency_schema_failures = validate_dependent_schemas(
            &entries,
            table_value,
            accessors,
            &keys,
            current_schema,
            schema_context,
            common_rules,
            &mut evaluated_locations,
        )
        .await;
    }

    let mut dependent_schema_failures = HashMap::new();
    if let Some(dependent_schemas) = &table_schema.dependent_schemas {
        let entries = dependent_schemas
            .iter()
            .map(|(key, schema_item)| (key.as_str(), schema_item))
            .collect_vec();
        dependent_schema_failures = validate_dependent_schemas(
            &entries,
            table_value,
            accessors,
            &keys,
            current_schema,
            schema_context,
            common_rules,
            &mut evaluated_locations,
        )
        .await;
    }

    let if_then_else_error = if let Some(if_then_else_schema) = &table_schema.if_then_else {
        let result = validate_if_then_else(
            table_value,
            accessors,
            if_then_else_schema,
            current_schema,
            schema_context,
            common_rules,
        )
        .await;
        evaluated_locations.merge_result_keeping_annotations(result)
    } else {
        None
    };
    let evaluated_properties = &evaluated_locations.properties;

    for (key, value) in table_value.key_values() {
        let key_rules = get_tombi_key_rules_and_diagnostics(key.comment_directives())
            .await
            .0;
        let key_common_rules = key_rules.as_ref().map(|rules| &rules.common);
        let key_rules = key_rules.as_ref().map(|rules| &rules.value);

        let accessor_text = key.value();
        let accessor = Accessor::Key(accessor_text.to_owned());
        let new_accessors = accessors
            .iter()
            .cloned()
            .chain(std::iter::once(Accessor::Key(accessor_text.to_owned())))
            .collect_vec();

        let mut matched_key = false;
        let mut declared_schema_applied = false;
        let mut declared_value_matched = true;
        let mut child_match_evidence = Box::<crate::MatchEvidence>::default();
        let schema_accessor = SchemaAccessor::from(&accessor);
        if table_schema
            .properties
            .read()
            .await
            .contains_key(&schema_accessor)
        {
            matched_key = true;

            match table_schema
                .resolve_property_schema(
                    &schema_accessor,
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    current_schema.strict,
                    schema_context.store,
                )
                .await
            {
                Ok(Some(current_schema)) => {
                    let result = value
                        .validate(&new_accessors, Some(&current_schema), schema_context)
                        .await;
                    declared_schema_applied = true;
                    declared_value_matched &= crate::validate::is_assertion_success(&result);
                    child_match_evidence
                        .merge_from(crate::validate::match_evidence(&result).clone());

                    if let Err(crate::Invalid {
                        assertion_failed: child_assertion_failed,
                        mut diagnostics,
                        ..
                    }) = result
                    {
                        assertion_failed |= child_assertion_failed;
                        convert_deprecated_diagnostics_range(
                            &current_schema,
                            value,
                            key,
                            &mut diagnostics,
                        )
                        .await;

                        total_diagnostics.extend(diagnostics);
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    if let Some(diagnostic) = crate::validate::schema_resolution_diagnostic(
                        &err,
                        key.range() + value.range(),
                        key_common_rules,
                    ) {
                        total_diagnostics.push(diagnostic);
                    }
                }
            }
        }

        if let Some(pattern_properties) = &table_schema.pattern_properties {
            let pattern_keys = pattern_properties
                .read()
                .await
                .keys()
                .cloned()
                .collect_vec();
            for pattern_key in pattern_keys {
                let Ok(pattern) = tombi_regex::Regex::new(&pattern_key) else {
                    log::warn!("invalid regex pattern property: {}", pattern_key);
                    continue;
                };
                if pattern.is_match(accessor_text) {
                    matched_key = true;
                    match table_schema
                        .resolve_pattern_property_schema(
                            &pattern_key,
                            current_schema.schema_uri.clone(),
                            current_schema.definitions.clone(),
                            current_schema.strict,
                            schema_context.store,
                        )
                        .await
                    {
                        Ok(Some(current_schema)) => {
                            let result = value
                                .validate(&new_accessors, Some(&current_schema), schema_context)
                                .await;
                            declared_schema_applied = true;
                            declared_value_matched &=
                                crate::validate::is_assertion_success(&result);
                            child_match_evidence
                                .merge_from(crate::validate::match_evidence(&result).clone());

                            if let Err(crate::Invalid {
                                assertion_failed: child_assertion_failed,
                                mut diagnostics,
                                ..
                            }) = result
                            {
                                assertion_failed |= child_assertion_failed;
                                convert_deprecated_diagnostics_range(
                                    &current_schema,
                                    value,
                                    key,
                                    &mut diagnostics,
                                )
                                .await;

                                total_diagnostics.extend(diagnostics);
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            if let Some(diagnostic) = crate::validate::schema_resolution_diagnostic(
                                &err,
                                key.range() + value.range(),
                                key_common_rules,
                            ) {
                                total_diagnostics.push(diagnostic);
                            }
                        }
                    }
                }
            }

            if !matched_key
                && !table_schema
                    .allows_additional_properties(schema_context.strict(Some(current_schema)))
            {
                assertion_failed = true;
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
                .push_diagnostic_with_level(level, &mut total_diagnostics);
            } else if key_rules
                .and_then(|rules| rules.key_pattern.as_ref())
                .and_then(|rules| rules.disabled)
                == Some(true)
            {
                handle_unused_noqa(
                    &mut total_diagnostics,
                    table_value.comment_directives(),
                    table_rules.as_ref().map(|rules| &rules.common),
                    "key-pattern",
                );
            }
        }

        if matched_key {
            match_evidence.merge_descendant_from(&child_match_evidence);
            if declared_schema_applied {
                if declared_value_matched && child_match_evidence.root_singleton_matched() {
                    match_evidence.mark_primary_value(new_accessors.clone());
                }
                match_evidence.mark_declared_child(new_accessors.clone(), declared_value_matched);
            }
        }

        if !matched_key {
            let mut validated_by_additional_schema = false;
            if let Some((_, referable_additional_property_schema)) =
                &table_schema.additional_property_schema
            {
                match tombi_schema_store::resolve_schema_item(
                    referable_additional_property_schema,
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    current_schema.strict,
                    schema_context.store,
                )
                .await
                {
                    Ok(Some(current_schema)) => {
                        let deprecation = current_schema.schema_view.deprecation().await;
                        handle_deprecated_value(
                            &mut total_diagnostics,
                            deprecation.as_ref(),
                            &new_accessors,
                            value,
                            Some(&current_schema),
                            schema_context,
                            table_value.comment_directives(),
                            table_rules.as_ref().map(|rules| &rules.common),
                        );

                        let result = value
                            .validate(&new_accessors, Some(&current_schema), schema_context)
                            .await;
                        if crate::validate::is_assertion_success(&result) {
                            match_evidence.mark_fallback_child_value(new_accessors.clone());
                        }
                        match_evidence
                            .merge_descendant_from(crate::validate::match_evidence(&result));
                        if let Err(crate::Invalid {
                            assertion_failed: child_assertion_failed,
                            diagnostics,
                            ..
                        }) = result
                        {
                            assertion_failed |= child_assertion_failed;
                            total_diagnostics.extend(diagnostics);
                        }
                        validated_by_additional_schema = true;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        if let Some(diagnostic) = crate::validate::schema_resolution_diagnostic(
                            &err,
                            key.range() + value.range(),
                            key_common_rules,
                        ) {
                            total_diagnostics.push(diagnostic);
                        }
                    }
                }
            }

            // `additionalProperties` contributes to evaluated properties only when the keyword exists.
            // When it's absent, unevaluatedProperties must still run.
            let evaluated_by_additional_default = table_schema.additional_properties().is_some();

            if !evaluated_properties.contains(accessor_text)
                && !validated_by_additional_schema
                && !evaluated_by_additional_default
            {
                if let Some(schema_item) = &table_schema.unevaluated_property_schema {
                    match tombi_schema_store::resolve_schema_item(
                        schema_item,
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        current_schema.strict,
                        schema_context.store,
                    )
                    .await
                    {
                        Ok(Some(unevaluated_schema)) => {
                            if let Err(crate::Invalid {
                                assertion_failed: child_assertion_failed,
                                diagnostics,
                                ..
                            }) = value
                                .validate(&new_accessors, Some(&unevaluated_schema), schema_context)
                                .await
                            {
                                assertion_failed |= child_assertion_failed;
                                total_diagnostics.extend(diagnostics);
                            }
                            continue;
                        }
                        Ok(None) => {}
                        Err(err) => {
                            if let Some(diagnostic) = crate::validate::schema_resolution_diagnostic(
                                &err,
                                key.range() + value.range(),
                                key_common_rules,
                            ) {
                                total_diagnostics.push(diagnostic);
                            }
                        }
                    }
                }

                if table_schema.unevaluated_properties == Some(false) {
                    assertion_failed = true;
                    crate::Diagnostic {
                        kind: Box::new(crate::DiagnosticKind::UnevaluatedPropertyNotAllowed {
                            key: key.to_string(),
                        }),
                        range: key.range() + value.range(),
                    }
                    .push_diagnostic_with_level(
                        SeverityLevelDefaultError::default(),
                        &mut total_diagnostics,
                    );
                    continue;
                }
            }

            if evaluated_properties.contains(accessor_text)
                && table_schema.additional_properties() != Some(false)
            {
                continue;
            }

            if table_schema.check_strict_additional_properties_violation(
                schema_context.strict(Some(current_schema)),
            ) {
                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::TableStrictAdditionalKeys {
                        accessors: MarkdownSchemaAccessors::from(accessors),
                        schema_uri: current_schema.schema_uri.as_ref().clone(),
                        key: key.to_string(),
                    }),
                    range: key.range() + value.range(),
                }
                .push_diagnostic_with_level(SeverityLevel::Warn, &mut total_diagnostics);

                continue;
            }
            if !table_schema
                .allows_any_additional_properties(schema_context.strict(Some(current_schema)))
            {
                assertion_failed = true;
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
                .push_diagnostic_with_level(level, &mut total_diagnostics);
                continue;
            } else if schema_context.strict(Some(current_schema))
                && key_rules
                    .and_then(|rules| rules.key_not_allowed.as_ref())
                    .and_then(|rules| rules.disabled)
                    == Some(true)
            {
                handle_unused_noqa(
                    &mut total_diagnostics,
                    table_value.comment_directives(),
                    table_rules.as_ref().map(|rules| &rules.common),
                    "key-not-allowed",
                );
            }
        }
    }

    if let Some(required) = &table_schema.required {
        for required_key in required {
            if !keys.contains(&required_key.as_str()) {
                assertion_failed = true;
                let level = table_rules
                    .map(|rules| &rules.value)
                    .and_then(|rules| {
                        rules
                            .table_key_required
                            .as_ref()
                            .map(SeverityLevelDefaultError::from)
                    })
                    .unwrap_or_default();

                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::TableKeyRequired {
                        key: required_key.to_string(),
                    }),
                    range: table_value.range(),
                }
                .push_diagnostic_with_level(level, &mut total_diagnostics);
            } else {
                if table_rules
                    .map(|rules| &rules.value)
                    .and_then(|rules| rules.table_key_required.as_ref())
                    .and_then(|rules| rules.disabled)
                    == Some(true)
                {
                    handle_unused_noqa(
                        &mut total_diagnostics,
                        table_value.comment_directives(),
                        table_rules.as_ref().map(|rules| &rules.common),
                        "table-key-required",
                    );
                }
                let path = accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(Accessor::Key(required_key.to_string())))
                    .collect();
                match_evidence.mark_required(path);
            }
        }
    }

    if let Some(max_properties) = table_schema.max_properties
        && table_value.keys().count() > max_properties
    {
        assertion_failed = true;
        let level = table_rules
            .map(|rules| &rules.value)
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
        .push_diagnostic_with_level(level, &mut total_diagnostics);
    } else if table_rules
        .map(|rules| &rules.value)
        .and_then(|rules| rules.table_max_keys.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut total_diagnostics,
            table_value.comment_directives(),
            table_rules.as_ref().map(|rules| &rules.common),
            "table-max-keys",
        );
    }

    if let Some(min_properties) = table_schema.min_properties
        && table_value.keys().count() < min_properties
    {
        assertion_failed = true;
        let level = table_rules
            .map(|rules| &rules.value)
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
        .push_diagnostic_with_level(level, &mut total_diagnostics);
    } else if table_rules
        .map(|rules| &rules.value)
        .and_then(|rules| rules.table_min_keys.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(
            &mut total_diagnostics,
            table_value.comment_directives(),
            table_rules.as_ref().map(|rules| &rules.common),
            "table-min-keys",
        );
    }

    if let Some(dependencies) = &table_schema.dependencies {
        for (dependent_key, dependency) in dependencies {
            match dependency {
                tombi_schema_store::Dependency::Property(required_keys) => {
                    if !keys.contains(&dependent_key.as_str()) {
                        continue;
                    }

                    for required_key in required_keys {
                        if !keys.contains(&required_key.as_str()) {
                            assertion_failed = true;
                            crate::Diagnostic {
                                kind: Box::new(crate::DiagnosticKind::TableDependencyRequired {
                                    dependent_key: dependent_key.to_string(),
                                    required_key: required_key.to_string(),
                                }),
                                range: table_value.range(),
                            }
                            .push_diagnostic_with_level(
                                SeverityLevelDefaultError::default(),
                                &mut total_diagnostics,
                            );
                        }
                    }
                }
                tombi_schema_store::Dependency::Schema(_) => {
                    if let Some(failure) = dependency_schema_failures.remove(dependent_key.as_str())
                    {
                        assertion_failed |= failure.assertion_failed;
                        total_diagnostics.extend(failure.diagnostics);
                    }
                }
            }
        }
    }

    if let Some(dependent_required) = &table_schema.dependent_required {
        for (dependent_key, required_keys) in dependent_required {
            if !keys.contains(&dependent_key.as_str()) {
                continue;
            }

            for required_key in required_keys {
                if !keys.contains(&required_key.as_str()) {
                    assertion_failed = true;
                    crate::Diagnostic {
                        kind: Box::new(crate::DiagnosticKind::TableDependencyRequired {
                            dependent_key: dependent_key.to_string(),
                            required_key: required_key.to_string(),
                        }),
                        range: table_value.range(),
                    }
                    .push_diagnostic_with_level(
                        SeverityLevelDefaultError::default(),
                        &mut total_diagnostics,
                    );
                }
            }
        }
    }

    if let Some(dependent_schemas) = &table_schema.dependent_schemas {
        // Walk the schema's own (insertion-ordered) map rather than draining the
        // failures, so diagnostics keep the order the keywords are declared in.
        for dependent_key in dependent_schemas.keys() {
            if let Some(error) = dependent_schema_failures.remove(dependent_key.as_str()) {
                assertion_failed |= error.assertion_failed;
                total_diagnostics.extend(error.diagnostics);
            }
        }
    }

    if table_schema.const_value.is_some() || table_schema.r#enum.is_some() {
        let actual_object = crate::convert::table_to_json_object(table_value);

        if let Some(const_value) = &table_schema.const_value {
            let matched = actual_object == *const_value;
            match_evidence.mark_root_value_assertion(matched, true);
            if !matched {
                assertion_failed = true;
                let level = table_rules
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
                        expected: tombi_json_value::Value::Object(const_value.clone()).to_string(),
                        actual: tombi_json_value::Value::Object(actual_object.clone()).to_string(),
                    }),
                    range: table_value.range(),
                }
                .push_diagnostic_with_level(level, &mut total_diagnostics);
            }
        } else if table_rules
            .and_then(|rules| rules.common.const_value.as_ref())
            .and_then(|rules| rules.disabled)
            == Some(true)
        {
            handle_unused_noqa(
                &mut total_diagnostics,
                table_value.comment_directives(),
                table_rules.as_ref().map(|rules| &rules.common),
                "const-value",
            );
        }

        if let Some(r#enum) = &table_schema.r#enum {
            let matched = r#enum.contains(&actual_object);
            match_evidence.mark_root_value_assertion(matched, r#enum.len() == 1);
            if !matched {
                assertion_failed = true;
                let level = table_rules
                    .map(|rules| &rules.common)
                    .and_then(|rules| rules.r#enum().map(SeverityLevelDefaultError::from))
                    .unwrap_or_default();

                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::Enum {
                        expected: r#enum
                            .iter()
                            .map(|item| tombi_json_value::Value::Object(item.clone()).to_string())
                            .collect(),
                        actual: tombi_json_value::Value::Object(actual_object).to_string(),
                    }),
                    range: table_value.range(),
                }
                .push_diagnostic_with_level(level, &mut total_diagnostics);
            }
        } else if table_rules
            .and_then(|rules| rules.common.r#enum())
            .and_then(|rules| rules.disabled)
            == Some(true)
        {
            handle_unused_noqa(
                &mut total_diagnostics,
                table_value.comment_directives(),
                table_rules.as_ref().map(|rules| &rules.common),
                "enum",
            );
        }
    }

    let property_name_current_schema =
        if let Some(property_name_schema) = &table_schema.property_names {
            match tombi_schema_store::resolve_schema_item(
                property_name_schema,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                current_schema.strict,
                schema_context.store,
            )
            .await
            {
                Ok(property_name_current_schema) => property_name_current_schema,
                Err(err) => {
                    if let Some(diagnostic) = crate::validate::schema_resolution_diagnostic(
                        &err,
                        table_value.range(),
                        common_rules,
                    ) {
                        total_diagnostics.push(diagnostic);
                    }
                    None
                }
            }
        } else {
            None
        };

    for key in table_value.keys() {
        if table_schema.property_names.is_some() && property_name_current_schema.is_none() {
            continue;
        }
        if let Err(crate::Invalid {
            assertion_failed: child_assertion_failed,
            diagnostics,
            ..
        }) = key
            .validate(
                accessors,
                property_name_current_schema.as_ref(),
                schema_context,
            )
            .await
        {
            assertion_failed |= child_assertion_failed;
            total_diagnostics.extend(diagnostics);
        }
    }

    if total_diagnostics.is_empty() && table_schema.deprecation.is_some() {
        handle_deprecated(
            &mut total_diagnostics,
            table_schema.deprecation.as_ref(),
            accessors,
            table_value,
            Some(current_schema),
            schema_context,
            table_value.comment_directives(),
            table_rules.as_ref().map(|rules| &rules.common),
        );
    }

    if let Some(error) = if_then_else_error {
        assertion_failed |= error.assertion_failed;
        match_evidence.merge_from(*error.match_evidence);
        total_diagnostics.extend(error.diagnostics);
    }

    let comment_directives = table_value
        .comment_directives()
        .map(|directives| directives.cloned().collect_vec());
    evaluated_locations.match_evidence = match_evidence.clone();
    let base_result = if total_diagnostics.is_empty() && !assertion_failed {
        Ok(evaluated_locations)
    } else {
        Err(crate::Invalid {
            assertion_failed,
            match_evidence,
            diagnostics: total_diagnostics,
            local_evaluated_locations: evaluated_locations,
        })
    };

    merge_validation_results(
        base_result,
        validate_adjacent_applicators(
            table_value,
            accessors,
            table_schema.one_of.as_deref(),
            table_schema.any_of.as_deref(),
            table_schema.all_of.as_deref(),
            table_schema.not.as_deref(),
            current_schema,
            schema_context,
            comment_directives.as_deref(),
            table_rules.map(|rules| &rules.common),
        )
        .await
        .or_else(filter_table_strict_additional_diagnostics),
    )
}

fn collect_evaluated_properties_from_table_schema<'a>(
    table_value: &'a tombi_document_tree_syntax::Table,
    accessors: &'a [tombi_schema_store::Accessor],
    table_schema: &'a tombi_schema_store::TableSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> BoxFuture<'a, crate::Valid> {
    async move {
        let mut result = crate::Valid::new();
        let property_keys = table_schema
            .properties
            .read()
            .await
            .keys()
            .filter_map(|accessor| match accessor {
                SchemaAccessor::Key(key) => Some(key.clone()),
                _ => None,
            })
            .collect_vec();
        for key in &property_keys {
            result.mark_property(key.clone());
        }

        let pattern_keys = if let Some(pattern_properties) = &table_schema.pattern_properties {
            pattern_properties
                .read()
                .await
                .keys()
                .cloned()
                .collect_vec()
        } else {
            Vec::new()
        };
        let pattern_regexes = pattern_keys
            .iter()
            .filter_map(|pattern_key| match tombi_regex::Regex::new(pattern_key) {
                Ok(pattern) => Some(pattern),
                Err(_) => {
                    log::warn!("invalid regex pattern property: {}", pattern_key);
                    None
                }
            })
            .collect_vec();

        for key in table_value.keys() {
            if property_keys
                .iter()
                .any(|property_key| property_key == key.value())
            {
                result.mark_property(key.value().to_string());
                continue;
            }

            if pattern_regexes
                .iter()
                .any(|pattern| pattern.is_match(key.value()))
            {
                result.mark_property(key.value().to_string());
            }
        }

        if table_schema.additional_properties().is_some() {
            for key in table_value.keys() {
                let matched_property = property_keys
                    .iter()
                    .any(|property_key| property_key == key.value());
                let matched_pattern = pattern_regexes
                    .iter()
                    .any(|pattern| pattern.is_match(key.value()));

                if !matched_property && !matched_pattern {
                    result.mark_property(key.value().to_string());
                }
            }
        }

        if let Some(one_of_schema) = &table_schema.one_of {
            result.merge_from(
                collect_evaluated_properties_from_referable_schemas(
                    table_value,
                    accessors,
                    one_of_schema.as_ref(),
                    current_schema,
                    schema_context,
                )
                .await,
            );
        }
        if let Some(any_of_schema) = &table_schema.any_of {
            result.merge_from(
                collect_evaluated_properties_from_referable_schemas(
                    table_value,
                    accessors,
                    any_of_schema.as_ref(),
                    current_schema,
                    schema_context,
                )
                .await,
            );
        }
        if let Some(all_of_schema) = &table_schema.all_of {
            result.merge_from(
                collect_evaluated_properties_from_referable_schemas(
                    table_value,
                    accessors,
                    all_of_schema.as_ref(),
                    current_schema,
                    schema_context,
                )
                .await,
            );
        }

        // An `unevaluatedProperties: true` subschema succeeds for every
        // remaining property and therefore annotates all of them as evaluated.
        // This annotation must be visible to adjacent outer applicators.
        if table_schema.unevaluated_property_schema.is_none()
            && table_schema.unevaluated_properties == Some(true)
        {
            for key in table_value.keys() {
                result.mark_property(key.value().to_string());
            }
        }

        result
    }
    .boxed()
}

fn collect_evaluated_properties_from_referable_schemas<'a>(
    table_value: &'a tombi_document_tree_syntax::Table,
    accessors: &'a [tombi_schema_store::Accessor],
    applicator: &'a (impl CompositeSchema + Sync),
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> BoxFuture<'a, crate::Valid> {
    async move {
        let mut result = crate::Valid::new();
        let Some(schemas) = tombi_schema_store::resolve_and_collect_schemas(
            applicator.schemas(),
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            current_schema.strict,
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
        )
        .await
        else {
            return result;
        };

        let comment_directives = table_value
            .comment_directives()
            .map(|directives| directives.cloned().collect_vec());

        for schema in &schemas {
            let Some(validation_result) = crate::validate::validate_resolved_schema(
                table_value,
                accessors,
                schema,
                schema_context,
                comment_directives.as_deref(),
                None,
            )
            .await
            else {
                continue;
            };

            if crate::validate::is_assertion_success(&validation_result) {
                match validation_result {
                    Ok(evaluated_locations) => result.merge_from(evaluated_locations),
                    Err(error) => result.merge_from(error.local_evaluated_locations),
                }
            }
        }
        result
    }
    .boxed()
}

async fn validate_table_without_schema(
    table_value: &tombi_document_tree_syntax::Table,
    accessors: &[tombi_schema_store::Accessor],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Result<crate::Valid, crate::Invalid> {
    let mut total_diagnostics = vec![];
    let mut assertion_failed = false;
    let mut match_evidence = Box::<crate::MatchEvidence>::default();
    let mut local_evaluated_locations = crate::Valid::new();

    // Validate without schema
    for (key, value) in table_value.key_values() {
        if let Err(crate::Invalid {
            assertion_failed: child_assertion_failed,
            match_evidence: child_match_evidence,
            diagnostics,
            local_evaluated_locations: child_evaluated_locations,
        }) = key.validate(accessors, None, schema_context).await
        {
            assertion_failed |= child_assertion_failed;
            match_evidence.merge_descendant_from(&child_match_evidence);
            local_evaluated_locations.merge_from(child_evaluated_locations);
            total_diagnostics.extend(diagnostics);
        }

        if let Err(crate::Invalid {
            assertion_failed: child_assertion_failed,
            match_evidence: child_match_evidence,
            diagnostics,
            local_evaluated_locations: child_evaluated_locations,
        }) = value
            .validate(
                &accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(Accessor::Key(key.value().to_owned())))
                    .collect_vec(),
                None,
                schema_context,
            )
            .await
        {
            assertion_failed |= child_assertion_failed;
            match_evidence.merge_descendant_from(&child_match_evidence);
            local_evaluated_locations.merge_from(child_evaluated_locations);
            total_diagnostics.extend(diagnostics);
        }
    }

    if total_diagnostics.is_empty() && !assertion_failed {
        Ok(local_evaluated_locations)
    } else {
        Err(crate::Invalid {
            assertion_failed,
            match_evidence,
            diagnostics: total_diagnostics,
            local_evaluated_locations,
        })
    }
}

/// Convert deprecated diagnostics to warnings for the given value
async fn convert_deprecated_diagnostics_range(
    current_schema: &CurrentSchema<'_>,
    value: &tombi_document_tree_syntax::Value,
    key: &tombi_document_tree_syntax::Key,
    schema_diagnostics: &mut [tombi_diagnostic::Diagnostic],
) {
    if current_schema.schema_view.deprecation().await.is_some() {
        for diagnostic in schema_diagnostics.iter_mut() {
            if diagnostic.code() == "deprecated" && diagnostic.range() == value.range() {
                let range = key.range() + value.range();
                *diagnostic = if diagnostic.is_error() {
                    tombi_diagnostic::Diagnostic::new_error(
                        diagnostic.message(),
                        diagnostic.code(),
                        range,
                    )
                } else {
                    tombi_diagnostic::Diagnostic::new_warning(
                        diagnostic.message(),
                        diagnostic.code(),
                        range,
                    )
                };
                break;
            }
        }
    }
}
