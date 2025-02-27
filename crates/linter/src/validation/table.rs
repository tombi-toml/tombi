use std::borrow::Cow;

use config::TomlVersion;
use document_tree::ValueImpl;
use futures::{future::BoxFuture, FutureExt};
use schema_store::{
    Accessor, CurrentSchema, SchemaAccessor, SchemaDefinitions, ValueSchema, ValueType,
};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};
use crate::error::Patterns;

impl Validate for document_tree::Table {
    fn validate<'a: 'b, 'b>(
        &'a self,
        toml_version: TomlVersion,
        accessors: &'a [Accessor],
        value_schema: Option<&'a ValueSchema>,
        schema_url: Option<&'a schema_store::SchemaUrl>,
        definitions: Option<&'a SchemaDefinitions>,
        sub_schema_url_map: &'a schema_store::SubSchemaUrlMap,
        schema_store: &'a schema_store::SchemaStore,
    ) -> BoxFuture<'b, Result<(), Vec<crate::Error>>> {
        tracing::trace!("self = {:?}", self);
        tracing::trace!("value_schema = {:?}", value_schema);

        async move {
            if let Some(sub_schema_url) = sub_schema_url_map.get(
                &accessors
                    .iter()
                    .map(SchemaAccessor::from)
                    .collect::<Vec<_>>(),
            ) {
                if schema_url != Some(sub_schema_url) {
                    if let Ok(document_schema) =
                        schema_store.try_load_document_schema(sub_schema_url).await
                    {
                        return self
                            .validate(
                                toml_version,
                                accessors,
                                document_schema.value_schema.as_ref(),
                                Some(&document_schema.schema_url),
                                Some(&document_schema.definitions),
                                sub_schema_url_map,
                                schema_store,
                            )
                            .await;
                    }
                }
            }

            let mut errors = vec![];
            match (value_schema, schema_url, definitions) {
                (Some(value_schema), Some(schema_url), Some(definitions)) => {
                    match value_schema.value_type().await {
                        ValueType::Table
                        | ValueType::OneOf(_)
                        | ValueType::AnyOf(_)
                        | ValueType::AllOf(_) => {}
                        ValueType::Null => return Ok(()),
                        value_schema => {
                            return Err(vec![crate::Error {
                                kind: crate::ErrorKind::TypeMismatch {
                                    expected: value_schema,
                                    actual: self.value_type(),
                                },
                                range: self.range(),
                            }])
                        }
                    }

                    let table_schema = match value_schema {
                        ValueSchema::Table(table_schema) => table_schema,
                        ValueSchema::OneOf(one_of_schema) => {
                            return validate_one_of(
                                self,
                                toml_version,
                                accessors,
                                one_of_schema,
                                schema_url,
                                definitions,
                                sub_schema_url_map,
                                schema_store,
                            )
                            .await
                        }
                        ValueSchema::AnyOf(any_of_schema) => {
                            return validate_any_of(
                                self,
                                toml_version,
                                accessors,
                                any_of_schema,
                                schema_url,
                                definitions,
                                sub_schema_url_map,
                                schema_store,
                            )
                            .await
                        }
                        ValueSchema::AllOf(all_of_schema) => {
                            return validate_all_of(
                                self,
                                toml_version,
                                accessors,
                                all_of_schema,
                                schema_url,
                                definitions,
                                sub_schema_url_map,
                                schema_store,
                            )
                            .await
                        }
                        _ => unreachable!("Expected a Table schema"),
                    };

                    for (key, value) in self.key_values() {
                        let accessor_raw_text = key.to_raw_text(toml_version);
                        let accessor = Accessor::Key(accessor_raw_text.clone());

                        let mut matche_key = false;
                        if let Some(property_schema) =
                            table_schema.properties.write().await.get_mut(&accessor)
                        {
                            matche_key = true;
                            match property_schema
                                .resolve(
                                    Cow::Borrowed(schema_url),
                                    Cow::Borrowed(definitions),
                                    schema_store,
                                )
                                .await
                            {
                                Ok(CurrentSchema {
                                    value_schema,
                                    schema_url,
                                    definitions,
                                }) => {
                                    if let Err(errs) = value
                                        .validate(
                                            toml_version,
                                            &accessors
                                                .iter()
                                                .cloned()
                                                .chain(std::iter::once(accessor.clone()))
                                                .collect::<Vec<_>>(),
                                            Some(value_schema),
                                            Some(&schema_url),
                                            Some(&definitions),
                                            sub_schema_url_map,
                                            schema_store,
                                        )
                                        .await
                                    {
                                        errors.extend(errs);
                                    }
                                }
                                Err(err) => {
                                    tracing::error!("{}", err);
                                }
                            }
                        }

                        if let Some(pattern_properties) = &table_schema.pattern_properties {
                            for (pattern_key, refferable_pattern_property) in
                                pattern_properties.write().await.iter_mut()
                            {
                                let Ok(pattern) = regex::Regex::new(pattern_key) else {
                                    tracing::error!(
                                        "Invalid regex pattern property: {}",
                                        pattern_key
                                    );
                                    continue;
                                };
                                if pattern.is_match(&accessor_raw_text) {
                                    matche_key = true;
                                    if let Ok(CurrentSchema {
                                        value_schema: pattern_property_schema,
                                        schema_url,
                                        definitions,
                                    }) = refferable_pattern_property
                                        .resolve(
                                            Cow::Borrowed(schema_url),
                                            Cow::Borrowed(definitions),
                                            schema_store,
                                        )
                                        .await
                                    {
                                        if let Err(errs) = value
                                            .validate(
                                                toml_version,
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor.clone()))
                                                    .collect::<Vec<_>>(),
                                                Some(pattern_property_schema),
                                                Some(&schema_url),
                                                Some(&definitions),
                                                sub_schema_url_map,
                                                schema_store,
                                            )
                                            .await
                                        {
                                            errors.extend(errs);
                                        }
                                    }
                                } else if !table_schema.additional_properties {
                                    errors.push(crate::Error {
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
                                    });
                                }
                            }
                        }
                        if !matche_key {
                            if let Some(referable_additional_property_schema) =
                                &table_schema.additional_property_schema
                            {
                                let mut referable_schema =
                                    referable_additional_property_schema.write().await;
                                if let Ok(CurrentSchema {
                                    value_schema: additional_property_schema,
                                    schema_url,
                                    definitions,
                                }) = referable_schema
                                    .resolve(
                                        Cow::Borrowed(schema_url),
                                        Cow::Borrowed(definitions),
                                        schema_store,
                                    )
                                    .await
                                {
                                    if let Err(errs) = value
                                        .validate(
                                            toml_version,
                                            &accessors
                                                .iter()
                                                .cloned()
                                                .chain(std::iter::once(accessor))
                                                .collect::<Vec<_>>(),
                                            Some(additional_property_schema),
                                            Some(&schema_url),
                                            Some(&definitions),
                                            sub_schema_url_map,
                                            schema_store,
                                        )
                                        .await
                                    {
                                        errors.extend(errs);
                                    }
                                }
                                continue;
                            }
                            if !table_schema.additional_properties {
                                errors.push(crate::Error {
                                    kind: crate::ErrorKind::KeyNotAllowed {
                                        key: key.to_string(),
                                    },
                                    range: key.range() + value.range(),
                                });
                                continue;
                            }
                        }
                    }

                    if let Some(required) = &table_schema.required {
                        let keys = self
                            .keys()
                            .map(|key| key.to_raw_text(toml_version))
                            .collect::<Vec<_>>();

                        for required_key in required {
                            if !keys.contains(required_key) {
                                errors.push(crate::Error {
                                    kind: crate::ErrorKind::KeyRequired {
                                        key: required_key.to_string(),
                                    },
                                    range: self.range(),
                                });
                            }
                        }
                    }

                    if let Some(max_properties) = table_schema.max_properties {
                        if self.keys().count() > max_properties {
                            errors.push(crate::Error {
                                kind: crate::ErrorKind::MaxProperties {
                                    max_properties,
                                    actual: self.keys().count(),
                                },
                                range: self.range(),
                            });
                        }
                    }

                    if let Some(min_properties) = table_schema.min_properties {
                        if self.keys().count() < min_properties {
                            errors.push(crate::Error {
                                kind: crate::ErrorKind::MinProperties {
                                    min_properties,
                                    actual: self.keys().count(),
                                },
                                range: self.range(),
                            });
                        }
                    }
                }
                _ => {
                    for (key, value) in self.key_values() {
                        if let Err(errs) = value
                            .validate(
                                toml_version,
                                &accessors
                                    .iter()
                                    .cloned()
                                    .chain(std::iter::once(Accessor::Key(
                                        key.to_raw_text(toml_version),
                                    )))
                                    .collect::<Vec<_>>(),
                                None,
                                None,
                                None,
                                sub_schema_url_map,
                                schema_store,
                            )
                            .await
                        {
                            errors.extend(errs);
                        }
                    }
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
        .boxed()
    }
}
