use std::borrow::Cow;

use ahash::AHashSet;
use itertools::Itertools;
use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::{LiteralValueRef, ValueImpl};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, DocumentSchema, ValueSchema, ValueType};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::Array {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            if let Some(sub_schema_url) = schema_context
                .sub_schema_url_map
                .and_then(|map| map.get(accessors))
            {
                if current_schema
                    .is_some_and(|current_schema| &*current_schema.schema_url != sub_schema_url)
                {
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
                    ValueType::Array
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

                let array_schema = match current_schema.value_schema.as_ref() {
                    ValueSchema::Array(array_schema) => array_schema,
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
                    _ => unreachable!("Expected an Array schema"),
                };

                if let Some(items) = &array_schema.items {
                    let mut referable_schema = items.write().await;
                    if let Ok(Some(current_schema)) = referable_schema
                        .resolve(
                            current_schema.schema_url.clone(),
                            current_schema.definitions.clone(),
                            schema_context.store,
                        )
                        .await
                    {
                        let new_accessors = accessors
                            .iter()
                            .cloned()
                            .chain(std::iter::once(tombi_schema_store::SchemaAccessor::Index))
                            .collect::<Vec<_>>();
                        if current_schema.value_schema.deprecated().await == Some(true) {
                            crate::Warning {
                                kind: Box::new(crate::WarningKind::Deprecated(
                                    tombi_schema_store::SchemaAccessors::new(new_accessors.clone()),
                                )),
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                        }

                        for value in self.values().iter() {
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
                    if self.values().len() > max_items {
                        crate::Error {
                            kind: crate::ErrorKind::MaxValues {
                                max_values: max_items,
                                actual: self.values().len(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(min_items) = array_schema.min_items {
                    if self.values().len() < min_items {
                        crate::Error {
                            kind: crate::ErrorKind::MinValues {
                                min_values: min_items,
                                actual: self.values().len(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }
                if array_schema.unique_items == Some(true) {
                    let literal_values = self
                        .values()
                        .iter()
                        .filter_map(Option::<LiteralValueRef>::from)
                        .counts();

                    let duplicated_values = literal_values
                        .iter()
                        .filter_map(|(value, count)| if *count > 1 { Some(value) } else { None })
                        .collect::<AHashSet<_>>();

                    for value in self.values() {
                        if let Some(literal_value) = Option::<LiteralValueRef>::from(value) {
                            if duplicated_values.contains(&literal_value) {
                                crate::Error {
                                    kind: crate::ErrorKind::UniqueValues,
                                    range: value.range(),
                                }
                                .set_diagnostics(&mut diagnostics);
                            }
                        }
                    }
                }
            } else {
                for value in self.values().iter() {
                    if let Err(value_diagnostics) = value
                        .validate(
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(tombi_schema_store::SchemaAccessor::Index))
                                .collect::<Vec<_>>(),
                            None,
                            schema_context,
                        )
                        .await
                    {
                        diagnostics.extend(value_diagnostics);
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
