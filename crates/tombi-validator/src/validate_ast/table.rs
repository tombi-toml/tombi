use std::borrow::Cow;

use itertools::Itertools;
use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, CurrentSchema, DocumentSchema, PropertySchema, SchemaAccessor, SchemaAccessors,
    ValueType,
};

use crate::{
    error::Patterns,
    validate_ast::{
        all_of::validate_all_of, any_of::validate_any_of, one_of::validate_one_of, type_mismatch,
        Validate, ValueImpl,
    },
};

impl Validate for tombi_ast::Table {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = Vec::new();

            // Validate all key-value pairs in the table
            for key_value in self.key_values() {
                if let Err(mut errs) = key_value
                    .validate(accessors, current_schema, schema_context, comment_context)
                    .await
                {
                    diagnostics.append(&mut errs);
                }
            }

            if diagnostics.is_empty() {
                Ok(())
            } else {
                Err(diagnostics)
            }
        }
        .boxed()
    }
}

impl<T> Validate for (&[tombi_ast::Key], &T)
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let (keys, value) = *self;

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
                        return value
                            .validate(
                                accessors,
                                Some(&CurrentSchema {
                                    value_schema: Cow::Borrowed(&value_schema),
                                    schema_uri: Cow::Borrowed(&schema_uri),
                                    definitions: Cow::Borrowed(&definitions),
                                }),
                                schema_context,
                                comment_context,
                            )
                            .await;
                    }
                }
            }

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::Table(table_schema) => {
                        validate_table_schema(
                            keys,
                            value,
                            accessors,
                            table_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
                        validate_one_of(
                            value,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::AnyOf(any_of_schema) => {
                        validate_any_of(
                            value,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::AllOf(all_of_schema) => {
                        validate_all_of(
                            value,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::Null => return Ok(()),
                    value_schema => {
                        type_mismatch(ValueType::Float, value.range(), value_schema).await
                    }
                }
            } else {
                validate_table_without_schema(
                    keys,
                    value,
                    accessors,
                    schema_context,
                    comment_context,
                )
                .await
            }
        }
        .boxed()
    }
}

fn validate_table_schema<'a: 'b, 'b, T>(
    keys: &'a [tombi_ast::Key],
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    table_schema: &'a tombi_schema_store::TableSchema,
    current_schema: &'a tombi_schema_store::CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext,
    comment_context: &'a CommentContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    async move {
        if let Some(key) = keys.first() {
            let Ok(key_raw_text) = key.try_to_raw_text(schema_context.toml_version) else {
                return (&keys[1..], value)
                    .validate(accessors, None, schema_context, comment_context)
                    .await;
            };

            let mut diagnostics = vec![];
            let accessor = Accessor::Key(key_raw_text.clone());
            let new_accessors = accessors
                .iter()
                .cloned()
                .chain(std::iter::once(Accessor::Key(key_raw_text.clone())))
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
                    if let Err(schema_diagnostics) = (&keys[1..], value)
                        .validate(
                            &new_accessors,
                            Some(&current_schema),
                            schema_context,
                            &comment_context,
                        )
                        .await
                    {
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
                        tracing::error!("Invalid regex pattern property: {}", pattern_key);
                        continue;
                    };
                    if pattern.is_match(&key_raw_text) {
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
                            if current_schema.value_schema.deprecated().await == Some(true) {
                                crate::Warning {
                                    kind: Box::new(crate::WarningKind::Deprecated(
                                        SchemaAccessors::from(&new_accessors),
                                    )),
                                    range: key.range() + value.range(),
                                }
                                .set_diagnostics(&mut diagnostics);
                            }
                            if let Err(schema_diagnostics) = (&keys[1..], value)
                                .validate(
                                    &new_accessors,
                                    Some(&current_schema),
                                    schema_context,
                                    &comment_context,
                                )
                                .await
                            {
                                diagnostics.extend(schema_diagnostics);
                            }
                        }
                    } else if !table_schema.allows_additional_properties(schema_context.strict()) {
                        crate::Error {
                            kind: crate::ErrorKind::KeyPattern {
                                patterns: Patterns(
                                    pattern_properties
                                        .read()
                                        .await
                                        .keys()
                                        .map(ToString::to_string)
                                        .collect(),
                                ),
                            },
                            range: key.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
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
                            crate::Warning {
                                kind: Box::new(crate::WarningKind::Deprecated(
                                    SchemaAccessors::from(&new_accessors),
                                )),
                                range: key.range() + value.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                        }

                        if let Err(schema_diagnostics) = (&keys[1..], value)
                            .validate(
                                &new_accessors,
                                Some(&current_schema),
                                schema_context,
                                &comment_context,
                            )
                            .await
                        {
                            diagnostics.extend(schema_diagnostics);
                        }
                    }
                }
                if table_schema
                    .check_strict_additional_properties_violation(schema_context.strict())
                {
                    crate::Warning {
                        kind: Box::new(crate::WarningKind::StrictAdditionalProperties {
                            accessors: SchemaAccessors::from(accessors),
                            schema_uri: current_schema.schema_uri.as_ref().clone(),
                            key: key.to_string(),
                        }),
                        range: key.range() + value.range(),
                    }
                    .set_diagnostics(&mut diagnostics);
                }
                if !table_schema.allows_any_additional_properties(schema_context.strict()) {
                    crate::Error {
                        kind: crate::ErrorKind::KeyNotAllowed {
                            key: key.to_string(),
                        },
                        range: key.range() + value.range(),
                    }
                    .set_diagnostics(&mut diagnostics);
                }
            }

            if diagnostics.is_empty() {
                Ok(())
            } else {
                Err(diagnostics)
            }
        } else {
            value
                .validate(
                    accessors,
                    Some(current_schema),
                    schema_context,
                    comment_context,
                )
                .await
        }
    }
    .boxed()
}

fn validate_table_without_schema<'a: 'b, 'b, T>(
    keys: &'a [tombi_ast::Key],
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_context: &'a CommentContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    async move {
        if keys.first().is_some() {
            (&keys[1..], value)
                .validate(accessors, None, schema_context, comment_context)
                .await
        } else {
            Ok(())
        }
    }
    .boxed()
}
