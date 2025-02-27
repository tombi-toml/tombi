use std::borrow::Cow;

use config::TomlVersion;
use document_tree::ValueImpl;
use futures::{future::BoxFuture, FutureExt};
use schema_store::{CurrentSchema, SchemaAccessor, SchemaDefinitions, ValueSchema, ValueType};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for document_tree::Array {
    fn validate<'a: 'b, 'b>(
        &'a self,
        toml_version: TomlVersion,
        accessors: &'a [schema_store::Accessor],
        value_schema: Option<&'a ValueSchema>,
        schema_url: Option<&'a schema_store::SchemaUrl>,
        definitions: Option<&'a SchemaDefinitions>,
        sub_schema_url_map: &'a schema_store::SubSchemaUrlMap,
        schema_store: &'a schema_store::SchemaStore,
    ) -> BoxFuture<'b, Result<(), Vec<crate::Error>>> {
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
                        ValueType::Array
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

                    let array_schema = match value_schema {
                        ValueSchema::Array(array_schema) => array_schema,
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
                        _ => unreachable!("Expected an Array schema"),
                    };

                    if let Some(items) = &array_schema.items {
                        let mut referable_schema = items.write().await;
                        if let Ok(CurrentSchema {
                            value_schema,
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
                            for (index, value) in self.values().iter().enumerate() {
                                if let Err(errs) = value
                                    .validate(
                                        toml_version,
                                        &accessors
                                            .iter()
                                            .cloned()
                                            .chain(std::iter::once(schema_store::Accessor::Index(
                                                index,
                                            )))
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
                        }
                    }

                    if let Some(max_items) = array_schema.max_items {
                        if self.values().len() > max_items {
                            errors.push(crate::Error {
                                kind: crate::ErrorKind::MaxItems {
                                    max_items,
                                    actual: self.values().len(),
                                },
                                range: self.range(),
                            });
                        }
                    }

                    if let Some(min_items) = array_schema.min_items {
                        if self.values().len() < min_items {
                            errors.push(crate::Error {
                                kind: crate::ErrorKind::MinItems {
                                    min_items,
                                    actual: self.values().len(),
                                },
                                range: self.range(),
                            });
                        }
                    }
                }
                _ => {
                    for (index, value) in self.values().iter().enumerate() {
                        if let Err(errs) = value
                            .validate(
                                toml_version,
                                &accessors
                                    .iter()
                                    .cloned()
                                    .chain(std::iter::once(schema_store::Accessor::Index(index)))
                                    .collect::<Vec<_>>(),
                                None,
                                schema_url,
                                definitions,
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
