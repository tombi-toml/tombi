use itertools::Itertools;
use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{Accessor, PropertySchema, SchemaAccessor, SchemaAccessors, ValueType};

use crate::{
    error::Patterns,
    header_accessor::HeaderAccessor,
    validate_ast::{type_mismatch, Validate, ValueImpl},
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
            let Some(header) = self.header() else {
                return Ok(());
            };

            let keys = header.keys().collect_vec();

            let mut total_diagnostics = vec![];

            for key_value in self.key_values() {
                if let Err(mut diagnostics) = (keys.as_slice(), &key_value)
                    .validate(accessors, current_schema, schema_context, comment_context)
                    .await
                {
                    total_diagnostics.append(&mut diagnostics);
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

pub fn validate_table_schema<'a: 'b, 'b, T>(
    header_accessors: &'a [HeaderAccessor],
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
        match header_accessors.first() {
            Some(HeaderAccessor::Key(key)) => {
                let Ok(key_raw_text) = key.try_to_raw_text(schema_context.toml_version) else {
                    return (&header_accessors[1..], value)
                        .validate(accessors, None, schema_context, comment_context)
                        .await;
                };

                let mut diagnostics = vec![];
                let accessor = Accessor::Key(key_raw_text.clone());
                let new_accessors = accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(accessor.clone()))
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
                        if let Err(schema_diagnostics) = (&header_accessors[1..], value)
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
                            tracing::warn!("Invalid regex pattern property: {}", pattern_key);
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
                                if let Err(schema_diagnostics) = (&header_accessors[1..], value)
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
                        } else if !table_schema
                            .allows_additional_properties(schema_context.strict())
                        {
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
                        let mut referable_schema =
                            referable_additional_property_schema.write().await;
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

                            if let Err(schema_diagnostics) = (&header_accessors[1..], value)
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
            }
            None => {
                value
                    .validate(
                        accessors,
                        Some(current_schema),
                        schema_context,
                        comment_context,
                    )
                    .await
            }
            Some(HeaderAccessor::Index { range, .. }) => {
                type_mismatch(ValueType::Table, ValueType::Array, *range)
            }
        }
    }
    .boxed()
}

pub fn validate_accessor_without_schema<'a: 'b, 'b, T>(
    header_accessors: &'a [HeaderAccessor],
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_context: &'a CommentContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    async move {
        if header_accessors.first().is_some() {
            (&header_accessors[1..], value)
                .validate(accessors, None, schema_context, comment_context)
                .await
        } else {
            Ok(())
        }
    }
    .boxed()
}
