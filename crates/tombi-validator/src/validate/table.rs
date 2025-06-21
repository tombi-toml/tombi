use std::borrow::Cow;

use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, CurrentSchema, DocumentSchema, PropertySchema, SchemaAccessor, SchemaAccessors,
    ValueSchema, ValueType,
};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};
use crate::error::Patterns;

impl Validate for tombi_document_tree::Table {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            if let Some(sub_schema_url) = schema_context
                .sub_schema_url_map
                .and_then(|map| map.get(accessors))
            {
                if current_schema.map(|schema| schema.schema_url.as_ref()) != Some(sub_schema_url) {
                    if let Ok(Some(DocumentSchema {
                        value_schema: Some(value_schema),
                        schema_url,
                        definitions,
                        ..
                    })) = schema_context
                        .store
                        .try_get_document_schema(sub_schema_url)
                        .await
                    {
                        return self
                            .validate(
                                accessors,
                                Some(&CurrentSchema {
                                    value_schema: Cow::Borrowed(&value_schema),
                                    schema_url: Cow::Borrowed(&schema_url),
                                    definitions: Cow::Borrowed(&definitions),
                                }),
                                schema_context,
                            )
                            .await;
                    }
                }
            }

            let mut diagnostics = vec![];
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.value_type().await {
                    ValueType::Table
                    | ValueType::OneOf(_)
                    | ValueType::AnyOf(_)
                    | ValueType::AllOf(_) => {}
                    ValueType::Null => return Ok(()),
                    value_schema => {
                        crate::Error {
                            kind: crate::ErrorKind::TypeMismatch {
                                expected: value_schema,
                                actual: self.value_type(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);

                        return Err(diagnostics);
                    }
                }

                let table_schema = match current_schema.value_schema.as_ref() {
                    ValueSchema::Table(table_schema) => table_schema,
                    ValueSchema::OneOf(one_of_schema) => {
                        return validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await;
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        return validate_any_of(
                            self,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await;
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        return validate_all_of(
                            self,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await;
                    }
                    _ => unreachable!("Expected a Table schema"),
                };

                for (key, value) in self.key_values() {
                    let accessor_raw_text = key.to_raw_text(schema_context.toml_version);
                    let accessor = Accessor::Key(accessor_raw_text.clone());
                    let new_accessors = accessors
                        .iter()
                        .cloned()
                        .chain(std::iter::once(SchemaAccessor::Key(
                            accessor_raw_text.clone(),
                        )))
                        .collect::<Vec<_>>();

                    let mut matche_key = false;
                    if let Some(PropertySchema {
                        property_schema, ..
                    }) = table_schema
                        .properties
                        .write()
                        .await
                        .get_mut(&SchemaAccessor::from(&accessor))
                    {
                        matche_key = true;
                        match property_schema
                            .resolve(
                                current_schema.schema_url.clone(),
                                current_schema.definitions.clone(),
                                schema_context.store,
                            )
                            .await
                        {
                            Ok(Some(current_schema)) => {
                                if current_schema.value_schema.deprecated().await == Some(true) {
                                    crate::Warning {
                                        kind: crate::WarningKind::Deprecated(SchemaAccessors::new(
                                            new_accessors.clone(),
                                        )),
                                        range: key.range() + value.range(),
                                    }
                                    .set_diagnostics(&mut diagnostics);
                                }
                                if let Err(schema_diagnostics) = value
                                    .validate(&new_accessors, Some(&current_schema), schema_context)
                                    .await
                                {
                                    diagnostics.extend(schema_diagnostics);
                                }
                            }
                            Ok(None) => {}
                            Err(err) => {
                                tracing::error!("{}", err);
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
                            if pattern.is_match(&accessor_raw_text) {
                                matche_key = true;
                                if let Ok(Some(current_schema)) = property_schema
                                    .resolve(
                                        current_schema.schema_url.clone(),
                                        current_schema.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                {
                                    if current_schema.value_schema.deprecated().await == Some(true)
                                    {
                                        crate::Warning {
                                            kind: crate::WarningKind::Deprecated(
                                                SchemaAccessors::new(new_accessors.clone()),
                                            ),
                                            range: key.range() + value.range(),
                                        }
                                        .set_diagnostics(&mut diagnostics);
                                    }
                                    if let Err(schema_diagnostics) = value
                                        .validate(
                                            &new_accessors,
                                            Some(&current_schema),
                                            schema_context,
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
                                    kind: crate::ErrorKind::PatternProperty {
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
                    if !matche_key {
                        if let Some((_, referable_additional_property_schema)) =
                            &table_schema.additional_property_schema
                        {
                            let mut referable_schema =
                                referable_additional_property_schema.write().await;
                            if let Ok(Some(current_schema)) = referable_schema
                                .resolve(
                                    current_schema.schema_url.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                if current_schema.value_schema.deprecated().await == Some(true) {
                                    crate::Warning {
                                        kind: crate::WarningKind::Deprecated(SchemaAccessors::new(
                                            new_accessors.clone(),
                                        )),
                                        range: key.range() + value.range(),
                                    }
                                    .set_diagnostics(&mut diagnostics);
                                }

                                if let Err(schema_diagnostics) = value
                                    .validate(&new_accessors, Some(&current_schema), schema_context)
                                    .await
                                {
                                    diagnostics.extend(schema_diagnostics);
                                }
                            }
                            continue;
                        }
                        if table_schema
                            .check_strict_additional_properties_violation(schema_context.strict())
                        {
                            crate::Warning {
                                kind: crate::WarningKind::StrictAdditionalProperties {
                                    key: key.to_string(),
                                },
                                range: key.range() + value.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                            continue;
                        }
                        if !table_schema.allows_any_additional_properties(schema_context.strict()) {
                            crate::Error {
                                kind: crate::ErrorKind::KeyNotAllowed {
                                    key: key.to_string(),
                                },
                                range: key.range() + value.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                            continue;
                        }
                    }
                }

                if let Some(required) = &table_schema.required {
                    let keys = self
                        .keys()
                        .map(|key| key.to_raw_text(schema_context.toml_version))
                        .collect::<Vec<_>>();

                    for required_key in required {
                        if !keys.contains(required_key) {
                            crate::Error {
                                kind: crate::ErrorKind::KeyRequired {
                                    key: required_key.to_string(),
                                },
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                        }
                    }
                }

                if let Some(max_properties) = table_schema.max_properties {
                    if self.keys().count() > max_properties {
                        crate::Error {
                            kind: crate::ErrorKind::MaxProperties {
                                max_properties,
                                actual: self.keys().count(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(min_properties) = table_schema.min_properties {
                    if self.keys().count() < min_properties {
                        crate::Error {
                            kind: crate::ErrorKind::MinProperties {
                                min_properties,
                                actual: self.keys().count(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }
            } else {
                for (key, value) in self.key_values() {
                    if let Err(schema_diagnostics) = value
                        .validate(
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(SchemaAccessor::Key(
                                    key.to_raw_text(schema_context.toml_version),
                                )))
                                .collect::<Vec<_>>(),
                            None,
                            schema_context,
                        )
                        .await
                    {
                        diagnostics.extend(schema_diagnostics);
                    }
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
