use std::borrow::Cow;

use ahash::AHashSet;
use itertools::Itertools;
use tombi_comment_directive::value::ArrayCommonLintRules;
use tombi_document_tree::{LiteralValueRef, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, DocumentSchema, ValueSchema};
use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};

use crate::{
    comment_directive::get_tombi_array_comment_directive_and_diagnostics, validate::type_mismatch,
};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::Array {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut total_diagnostics = vec![];
            let (value_rules, diagnostics) =
                get_tombi_array_comment_directive_and_diagnostics(self, accessors).await;

            if !diagnostics.is_empty() {
                total_diagnostics.extend(diagnostics);
            }

            if let Some(sub_schema_uri) = schema_context
                .sub_schema_uri_map
                .and_then(|map| map.get(&accessors.into_iter().map(Into::into).collect_vec()))
            {
                if current_schema
                    .is_some_and(|current_schema| &*current_schema.schema_uri != sub_schema_uri)
                {
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
                    ValueSchema::Array(array_schema) => {
                        validate_array(
                            self,
                            accessors,
                            array_schema,
                            current_schema,
                            schema_context,
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
            } else {
                if let Err(diagnostics) =
                    validate_array_without_schema(self, accessors, schema_context).await
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

async fn validate_array(
    array_value: &tombi_document_tree::Array,
    accessors: &[tombi_schema_store::Accessor],
    array_schema: &tombi_schema_store::ArraySchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    value_rules: Option<&ArrayCommonLintRules>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];

    if let Some(items) = &array_schema.items {
        let mut referable_schema = items.write().await;
        if let Ok(Some(current_schema)) = referable_schema
            .resolve(
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
            .inspect_err(|err| tracing::warn!("{err}"))
        {
            for (index, value) in array_value.values().iter().enumerate() {
                let new_accessors = accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(tombi_schema_store::Accessor::Index(index)))
                    .collect_vec();

                if let Err(schema_diagnostics) = value
                    .validate(&new_accessors, Some(&current_schema), schema_context)
                    .await
                {
                    diagnostics.extend(schema_diagnostics);
                }
            }
        }
    }

    if let Some(max_items) = array_schema.max_items {
        if array_value.values().len() > max_items {
            let level = value_rules
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
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if let Some(min_items) = array_schema.min_items {
        if array_value.values().len() < min_items {
            let level = value_rules
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
            .push_diagnostic_with_level(level, &mut diagnostics);
        }
    }

    if array_schema.unique_items == Some(true) {
        let level = value_rules
            .map(|rules| &rules.value)
            .and_then(|rules| {
                rules
                    .array_unique_values
                    .as_ref()
                    .map(SeverityLevelDefaultError::from)
            })
            .unwrap_or_default();

        let literal_values = array_value
            .values()
            .iter()
            .filter_map(Option::<LiteralValueRef>::from)
            .counts();

        let duplicated_values = literal_values
            .iter()
            .filter_map(|(value, count)| if *count > 1 { Some(value) } else { None })
            .collect::<AHashSet<_>>();

        for value in array_value.values() {
            if let Some(literal_value) = Option::<LiteralValueRef>::from(value) {
                if duplicated_values.contains(&literal_value) {
                    crate::Diagnostic {
                        kind: Box::new(crate::DiagnosticKind::ArrayUniqueValues),
                        range: value.range(),
                    }
                    .push_diagnostic_with_level(level, &mut diagnostics);
                }
            }
        }
    }

    if diagnostics.is_empty() {
        if array_schema.deprecated == Some(true) {
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
                kind: Box::new(crate::DiagnosticKind::Deprecated(
                    tombi_schema_store::SchemaAccessors::from(accessors),
                )),
                range: array_value.range(),
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

async fn validate_array_without_schema(
    array_value: &tombi_document_tree::Array,
    accessors: &[tombi_schema_store::Accessor],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];

    // Validate without schema
    for (index, value) in array_value.values().iter().enumerate() {
        if let Err(value_diagnostics) = value
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
            diagnostics.extend(value_diagnostics);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
