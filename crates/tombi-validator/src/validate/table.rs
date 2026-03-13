use itertools::Itertools;
use std::borrow::Cow;
use tombi_comment_directive::value::{
    StringCommonFormatRules, StringCommonLintRules, TableCommonLintRules,
};
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_hashmap::HashSet;
use tombi_schema_store::{
    Accessor, CompositeSchema, CurrentSchema, SchemaAccessor, SchemaAccessors, ValueSchema,
};
use tombi_severity_level::{SeverityLevel, SeverityLevelDefaultError, SeverityLevelDefaultWarn};

use crate::{
    comment_directive::{
        get_tombi_key_rules_and_diagnostics, get_tombi_key_table_value_rules_and_diagnostics,
        get_tombi_table_comment_directive_and_diagnostics,
    },
    error::{REQUIRED_KEY_SCORE, TYPE_MATCHED_SCORE},
    validate::{
        handle_anything_schema, handle_deprecated, handle_deprecated_value, handle_nothing_schema,
        handle_type_mismatch, handle_unused_noqa, if_then_else::validate_if_then_else,
        merge_validation_results, string::validate_raw_string, validate_adjacent_applicators,
    },
};

use super::{Validate, validate_all_of, validate_any_of, validate_one_of};
use crate::diagnostic::Patterns;

impl Validate for tombi_document_tree::Table {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), crate::Error>> {
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
                get_tombi_table_comment_directive_and_diagnostics(self, accessors).await;

            let result = if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Table(table_schema) => {
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
                validate_table_without_schema(self, accessors, schema_context).await
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

async fn validate_table(
    table_value: &tombi_document_tree::Table,
    accessors: &[tombi_schema_store::Accessor],
    table_schema: &tombi_schema_store::TableSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    table_rules: Option<&TableCommonLintRules>,
) -> Result<(), crate::Error> {
    let mut total_score = TYPE_MATCHED_SCORE;
    let mut total_diagnostics = vec![];
    let adjacent_allowed_keys =
        collect_adjacent_allowed_keys(table_schema, current_schema, schema_context).await;

    for (key, value) in table_value.key_values() {
        let key_rules = get_tombi_key_rules_and_diagnostics(key.comment_directives())
            .await
            .0
            .map(|rules| rules.value);

        let key_rules = key_rules.as_ref();

        let accessor_raw_text = &key.value;
        let accessor = Accessor::Key(accessor_raw_text.to_owned());
        let new_accessors = accessors
            .iter()
            .cloned()
            .chain(std::iter::once(Accessor::Key(accessor_raw_text.to_owned())))
            .collect_vec();

        let mut matched_key = false;
        let schema_accessor = SchemaAccessor::from(&accessor);
        if table_schema
            .properties
            .read()
            .await
            .contains_key(&schema_accessor)
        {
            matched_key = true;

            if let Ok(Some(current_schema)) = table_schema
                .resolve_property_schema(
                    &schema_accessor,
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| log::warn!("{err}"))
                && let Err(crate::Error {
                    mut diagnostics, ..
                }) = value
                    .validate(&new_accessors, Some(&current_schema), schema_context)
                    .await
            {
                convert_deprecated_diagnostics_range(&current_schema, value, key, &mut diagnostics)
                    .await;

                total_diagnostics.extend(diagnostics);
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
                    log::warn!("Invalid regex pattern property: {}", pattern_key);
                    continue;
                };
                if pattern.is_match(accessor_raw_text) {
                    matched_key = true;
                    if let Ok(Some(current_schema)) = table_schema
                        .resolve_pattern_property_schema(
                            &pattern_key,
                            current_schema.schema_uri.clone(),
                            current_schema.definitions.clone(),
                            schema_context.store,
                        )
                        .await
                        .inspect_err(|err| log::warn!("{err}"))
                        && let Err(crate::Error {
                            mut diagnostics, ..
                        }) = value
                            .validate(&new_accessors, Some(&current_schema), schema_context)
                            .await
                    {
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
            }

            if !matched_key && !table_schema.allows_additional_properties(schema_context.strict()) {
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

        if !matched_key {
            let mut validated_by_additional_schema = false;
            if let Some((_, referable_additional_property_schema)) =
                &table_schema.additional_property_schema
                && let Ok(Some(current_schema)) = tombi_schema_store::resolve_schema_item(
                    referable_additional_property_schema,
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| log::warn!("{err}"))
            {
                handle_deprecated_value(
                    &mut total_diagnostics,
                    current_schema.value_schema.deprecated().await,
                    &new_accessors,
                    value,
                    table_value.comment_directives(),
                    table_rules.as_ref().map(|rules| &rules.common),
                );

                if let Err(crate::Error { diagnostics, .. }) = value
                    .validate(&new_accessors, Some(&current_schema), schema_context)
                    .await
                {
                    total_diagnostics.extend(diagnostics);
                }
                validated_by_additional_schema = true;
            }

            // `additionalProperties` contributes to evaluated properties only when the keyword exists.
            // When it's absent, unevaluatedProperties must still run.
            let evaluated_by_additional_default = table_schema.additional_properties().is_some();

            if !validated_by_additional_schema && !evaluated_by_additional_default {
                if let Some(schema_item) = &table_schema.unevaluated_property_schema
                    && let Ok(Some(unevaluated_schema)) = tombi_schema_store::resolve_schema_item(
                        schema_item,
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                    )
                    .await
                    .inspect_err(|err| log::warn!("{err}"))
                {
                    if let Err(crate::Error { diagnostics, .. }) = value
                        .validate(&new_accessors, Some(&unevaluated_schema), schema_context)
                        .await
                    {
                        total_diagnostics.extend(diagnostics);
                    }
                    continue;
                }

                if table_schema.unevaluated_properties == Some(false) {
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
            if adjacent_allowed_keys.contains(accessor_raw_text) {
                continue;
            }

            if table_schema.check_strict_additional_properties_violation(schema_context.strict()) {
                crate::Diagnostic {
                    kind: Box::new(crate::DiagnosticKind::TableStrictAdditionalKeys {
                        accessors: SchemaAccessors::from(accessors),
                        schema_uri: current_schema.schema_uri.as_ref().clone(),
                        key: key.to_string(),
                    }),
                    range: key.range() + value.range(),
                }
                .push_diagnostic_with_level(SeverityLevel::Warn, &mut total_diagnostics);

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
                .push_diagnostic_with_level(level, &mut total_diagnostics);
                continue;
            } else if schema_context.strict()
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

    let keys = table_value.keys().map(|key| &key.value).collect_vec();

    if let Some(required) = &table_schema.required {
        for required_key in required {
            if !keys.contains(&required_key) {
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
                total_score += REQUIRED_KEY_SCORE;
            }
        }
    }

    if let Some(max_properties) = table_schema.max_properties
        && table_value.keys().count() > max_properties
    {
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
            if !keys.contains(&dependent_key) {
                continue;
            }

            match dependency {
                tombi_schema_store::Dependency::Property(required_keys) => {
                    for required_key in required_keys {
                        if !keys.contains(&required_key) {
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
                tombi_schema_store::Dependency::Schema(schema_item) => {
                    if let Ok(Some(dep_schema)) = tombi_schema_store::resolve_schema_item(
                        schema_item,
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                    )
                    .await
                    .inspect_err(|err| log::warn!("{err}"))
                    {
                        // A dependency schema is an additional constraint layered on top of
                        // the parent table schema. Running strict mode here against the
                        // partial dependency schema causes false-positive additional key
                        // diagnostics for valid keys defined by the parent schema.
                        let dependency_schema_context = tombi_schema_store::SchemaContext {
                            toml_version: schema_context.toml_version,
                            root_schema: schema_context.root_schema,
                            sub_schema_uri_map: schema_context.sub_schema_uri_map,
                            schema_visits: schema_context.schema_visits.clone(),
                            store: schema_context.store,
                            strict: Some(false),
                        };

                        if let Err(crate::Error { diagnostics, .. }) = table_value
                            .validate(accessors, Some(&dep_schema), &dependency_schema_context)
                            .await
                        {
                            total_diagnostics.extend(diagnostics);
                        }
                    }
                }
            }
        }
    }

    if let Some(dependent_required) = &table_schema.dependent_required {
        for (dependent_key, required_keys) in dependent_required {
            if !keys.contains(&dependent_key) {
                continue;
            }

            for required_key in required_keys {
                if !keys.contains(&required_key) {
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
        for (dependent_key, schema_item) in dependent_schemas {
            if !keys.contains(&dependent_key) {
                continue;
            }

            if let Ok(Some(dep_schema)) = tombi_schema_store::resolve_schema_item(
                schema_item,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
            .inspect_err(|err| log::warn!("{err}"))
            {
                // See the rationale in the `Dependency::Schema` branch above.
                let dependency_schema_context = tombi_schema_store::SchemaContext {
                    toml_version: schema_context.toml_version,
                    root_schema: schema_context.root_schema,
                    sub_schema_uri_map: schema_context.sub_schema_uri_map,
                    schema_visits: schema_context.schema_visits.clone(),
                    store: schema_context.store,
                    strict: Some(false),
                };

                if let Err(crate::Error { diagnostics, .. }) = table_value
                    .validate(accessors, Some(&dep_schema), &dependency_schema_context)
                    .await
                {
                    total_diagnostics.extend(diagnostics);
                }
            }
        }
    }

    if table_schema.const_value.is_some() || table_schema.r#enum.is_some() {
        let actual_object = crate::convert::table_to_json_object(table_value);

        if let Some(const_value) = &table_schema.const_value {
            if actual_object != *const_value {
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
            if !r#enum.contains(&actual_object) {
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

    if let Some(property_name_schema) = &table_schema.property_names
        && let Ok(Some(property_name_current_schema)) = tombi_schema_store::resolve_schema_item(
            property_name_schema,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
        )
        .await
        .inspect_err(|err| log::warn!("{err}"))
        && let ValueSchema::String(string_schema) =
            property_name_current_schema.value_schema.as_ref()
    {
        let allows_empty_key = string_schema.min_length == Some(0);

        let format_assertion = schema_context
            .root_schema
            .is_none_or(|root| root.format_assertion())
            || string_schema
                .format
                .is_some_and(|format| schema_context.has_string_format(format));

        for key in table_value.keys() {
            if !allows_empty_key {
                check_key_empty(key, &mut total_diagnostics).await;
            }

            let (lint_rules, lint_rules_diagnostics) =
                get_tombi_key_table_value_rules_and_diagnostics::<
                    StringCommonFormatRules,
                    StringCommonLintRules,
                >(key.comment_directives(), accessors)
                .await;

            let result = validate_raw_string(
                &key.value,
                &key.value,
                key.range(),
                string_schema,
                format_assertion,
                lint_rules.as_ref(),
                key.comment_directives(),
            );

            match result {
                Ok(()) => {
                    if !lint_rules_diagnostics.is_empty() {
                        total_diagnostics.extend(lint_rules_diagnostics);
                    }
                }
                Err(mut error) => {
                    error.prepend_diagnostics(lint_rules_diagnostics);
                    total_diagnostics.extend(error.diagnostics);
                }
            }
        }
    } else {
        for key in table_value.keys() {
            check_key_empty(key, &mut total_diagnostics).await;
        }
    }

    if total_diagnostics.is_empty() {
        handle_deprecated(
            &mut total_diagnostics,
            table_schema.deprecated,
            accessors,
            table_value,
            table_value.comment_directives(),
            table_rules.as_ref().map(|rules| &rules.common),
        );
    }

    if let Some(if_then_else_schema) = table_schema.if_then_else.as_ref()
        && let Err(error) = validate_if_then_else(
            table_value,
            accessors,
            if_then_else_schema,
            current_schema,
            schema_context,
        )
        .await
    {
        total_diagnostics.extend(error.diagnostics);
    }

    let comment_directives = table_value
        .comment_directives()
        .map(|directives| directives.cloned().collect_vec());
    let base_result = if total_diagnostics.is_empty() {
        Ok(())
    } else {
        Err(crate::Error {
            score: total_score,
            diagnostics: total_diagnostics,
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
        .await,
    )
}

async fn collect_adjacent_allowed_keys(
    table_schema: &tombi_schema_store::TableSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> HashSet<String> {
    let mut keys = HashSet::new();
    collect_keys_from_table_schema(table_schema, current_schema, schema_context, &mut keys).await;
    keys
}

fn collect_keys_from_table_schema<'a>(
    table_schema: &'a tombi_schema_store::TableSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    keys: &'a mut HashSet<String>,
) -> BoxFuture<'a, ()> {
    async move {
        for accessor in table_schema.properties.read().await.keys() {
            if let SchemaAccessor::Key(key) = accessor {
                keys.insert(key.clone());
            }
        }

        if let Some(dependencies) = &table_schema.dependencies {
            for (dependent_key, dependency) in dependencies {
                keys.insert(dependent_key.clone());
                match dependency {
                    tombi_schema_store::Dependency::Property(required_keys) => {
                        keys.extend(required_keys.iter().cloned());
                    }
                    tombi_schema_store::Dependency::Schema(schema_item) => {
                        collect_keys_from_schema_item(
                            schema_item,
                            current_schema,
                            schema_context,
                            keys,
                        )
                        .await;
                    }
                }
            }
        }

        if let Some(dependent_required) = &table_schema.dependent_required {
            for (dependent_key, required_keys) in dependent_required {
                keys.insert(dependent_key.clone());
                keys.extend(required_keys.iter().cloned());
            }
        }

        if let Some(dependent_schemas) = &table_schema.dependent_schemas {
            for (dependent_key, schema_item) in dependent_schemas {
                keys.insert(dependent_key.clone());
                collect_keys_from_schema_item(schema_item, current_schema, schema_context, keys)
                    .await;
            }
        }

        if let Some(one_of_schema) = &table_schema.one_of {
            collect_keys_from_referable_schemas(
                one_of_schema.as_ref(),
                current_schema,
                schema_context,
                keys,
            )
            .await;
        }
        if let Some(any_of_schema) = &table_schema.any_of {
            collect_keys_from_referable_schemas(
                any_of_schema.as_ref(),
                current_schema,
                schema_context,
                keys,
            )
            .await;
        }
        if let Some(all_of_schema) = &table_schema.all_of {
            collect_keys_from_referable_schemas(
                all_of_schema.as_ref(),
                current_schema,
                schema_context,
                keys,
            )
            .await;
        }
    }
    .boxed()
}

fn collect_keys_from_schema_item<'a>(
    schema_item: &'a tombi_schema_store::SchemaItem,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    keys: &'a mut HashSet<String>,
) -> BoxFuture<'a, ()> {
    async move {
        if let Ok(Some(schema)) = tombi_schema_store::resolve_schema_item(
            schema_item,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
        )
        .await
        .inspect_err(|err| log::warn!("{err}"))
        {
            collect_keys_from_value_schema(
                schema.value_schema.as_ref(),
                &schema,
                schema_context,
                keys,
            )
            .await;
        }
    }
    .boxed()
}

fn collect_keys_from_referable_schemas<'a>(
    applicator: &'a (impl CompositeSchema + Sync),
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    keys: &'a mut HashSet<String>,
) -> BoxFuture<'a, ()> {
    async move {
        for schema_item in applicator.schemas().read().await.iter() {
            if let Ok(Some(schema)) = schema_item
                .to_current_schema(
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| log::warn!("{err}"))
            {
                collect_keys_from_value_schema(
                    schema.value_schema.as_ref(),
                    &schema,
                    schema_context,
                    keys,
                )
                .await;
            }
        }
    }
    .boxed()
}

fn collect_keys_from_value_schema<'a>(
    value_schema: &'a ValueSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    keys: &'a mut HashSet<String>,
) -> BoxFuture<'a, ()> {
    async move {
        match value_schema {
            ValueSchema::Table(table_schema) => {
                collect_keys_from_table_schema(table_schema, current_schema, schema_context, keys)
                    .await;
            }
            ValueSchema::OneOf(one_of_schema) => {
                collect_keys_from_referable_schemas(
                    one_of_schema,
                    current_schema,
                    schema_context,
                    keys,
                )
                .await;
            }
            ValueSchema::AnyOf(any_of_schema) => {
                collect_keys_from_referable_schemas(
                    any_of_schema,
                    current_schema,
                    schema_context,
                    keys,
                )
                .await;
            }
            ValueSchema::AllOf(all_of_schema) => {
                collect_keys_from_referable_schemas(
                    all_of_schema,
                    current_schema,
                    schema_context,
                    keys,
                )
                .await;
            }
            _ => {}
        }
    }
    .boxed()
}

async fn validate_table_without_schema(
    table_value: &tombi_document_tree::Table,
    accessors: &[tombi_schema_store::Accessor],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Result<(), crate::Error> {
    let mut total_diagnostics = vec![];

    // Validate without schema
    for (key, value) in table_value.key_values() {
        check_key_empty(key, &mut total_diagnostics).await;

        if let Err(crate::Error { diagnostics, .. }) = value
            .validate(
                &accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(Accessor::Key(key.value.clone())))
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

async fn check_key_empty(
    key: &tombi_document_tree::Key,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if !key.value.is_empty() {
        return;
    }

    let (key_rules, key_rules_diagnostics) =
        get_tombi_key_rules_and_diagnostics(key.comment_directives()).await;
    let key_rules = key_rules.map(|rules| rules.value);

    diagnostics.extend(key_rules_diagnostics);

    let level = key_rules
        .as_ref()
        .and_then(|rules| rules.key_empty.as_ref().map(SeverityLevelDefaultWarn::from))
        .unwrap_or_default();

    crate::Diagnostic {
        kind: Box::new(crate::DiagnosticKind::KeyEmpty),
        range: key.range(),
    }
    .push_diagnostic_with_level(level, diagnostics);
}

/// Convert deprecated diagnostics to warnings for the given value
async fn convert_deprecated_diagnostics_range(
    current_schema: &CurrentSchema<'_>,
    value: &tombi_document_tree::Value,
    key: &tombi_document_tree::Key,
    schema_diagnostics: &mut [tombi_diagnostic::Diagnostic],
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
