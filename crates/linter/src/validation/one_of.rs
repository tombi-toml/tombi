use std::{borrow::Cow, fmt::Debug};

use config::TomlVersion;
use document_tree::ValueImpl;
use futures::{future::BoxFuture, FutureExt};
use schema_store::{CurrentSchema, OneOfSchema, SchemaDefinitions, ValueSchema};

use super::Validate;
use crate::validation::{all_of::validate_all_of, any_of::validate_any_of};

pub fn validate_one_of<'a: 'b, 'b, T>(
    value: &'a T,
    toml_version: TomlVersion,
    accessors: &'a [schema_store::Accessor],
    one_of_schema: &'a OneOfSchema,
    schema_url: &'a schema_store::SchemaUrl,
    definitions: &'a SchemaDefinitions,
    sub_schema_url_map: &'a schema_store::SubSchemaUrlMap,
    schema_store: &'a schema_store::SchemaStore,
) -> BoxFuture<'b, Result<(), Vec<crate::Error>>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    tracing::trace!("value = {:?}", value);
    tracing::trace!("one_of_schema = {:?}", one_of_schema);

    async move {
        let mut errors = vec![];
        let mut is_type_match = false;
        let mut type_mismatch_errors = vec![];
        let mut valid_count = 0;

        let mut schemas = one_of_schema.schemas.write().await;
        for referable_schema in schemas.iter_mut() {
            let Ok(CurrentSchema {
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
            else {
                continue;
            };

            match (value.value_type(), value_schema) {
                (document_tree::ValueType::Boolean, ValueSchema::Boolean(_))
                | (document_tree::ValueType::Integer, ValueSchema::Integer(_))
                | (document_tree::ValueType::Float, ValueSchema::Float(_))
                | (document_tree::ValueType::String, ValueSchema::String(_))
                | (document_tree::ValueType::OffsetDateTime, ValueSchema::OffsetDateTime(_))
                | (document_tree::ValueType::LocalDateTime, ValueSchema::LocalDateTime(_))
                | (document_tree::ValueType::LocalDate, ValueSchema::LocalDate(_))
                | (document_tree::ValueType::LocalTime, ValueSchema::LocalTime(_))
                | (document_tree::ValueType::Table, ValueSchema::Table(_))
                | (document_tree::ValueType::Array, ValueSchema::Array(_)) => {
                    is_type_match = true;
                    match value
                        .validate(
                            toml_version,
                            accessors,
                            Some(value_schema),
                            Some(&schema_url),
                            Some(&definitions),
                            sub_schema_url_map,
                            schema_store,
                        )
                        .await
                    {
                        Ok(()) => {
                            valid_count += 1;
                            break;
                        }
                        Err(mut schema_errors) => errors.append(&mut schema_errors),
                    }
                }
                (_, ValueSchema::Boolean(_))
                | (_, ValueSchema::Integer(_))
                | (_, ValueSchema::Float(_))
                | (_, ValueSchema::String(_))
                | (_, ValueSchema::OffsetDateTime(_))
                | (_, ValueSchema::LocalDateTime(_))
                | (_, ValueSchema::LocalDate(_))
                | (_, ValueSchema::LocalTime(_))
                | (_, ValueSchema::Table(_))
                | (_, ValueSchema::Array(_))
                | (_, ValueSchema::Null) => {
                    type_mismatch_errors.push(crate::Error {
                        kind: crate::ErrorKind::TypeMismatch {
                            expected: value_schema.value_type().await,
                            actual: value.value_type(),
                        },
                        range: value.range(),
                    });
                }
                (_, ValueSchema::OneOf(one_of_schema)) => {
                    match validate_one_of(
                        value,
                        toml_version,
                        accessors,
                        one_of_schema,
                        &schema_url,
                        &definitions,
                        sub_schema_url_map,
                        schema_store,
                    )
                    .await
                    {
                        Ok(()) => {
                            valid_count += 1;
                            break;
                        }
                        Err(mut schema_errors) => errors.append(&mut schema_errors),
                    }
                }
                (_, ValueSchema::AnyOf(any_of_schema)) => {
                    match validate_any_of(
                        value,
                        toml_version,
                        accessors,
                        any_of_schema,
                        &schema_url,
                        &definitions,
                        sub_schema_url_map,
                        schema_store,
                    )
                    .await
                    {
                        Ok(()) => {
                            valid_count += 1;
                            break;
                        }
                        Err(mut schema_errors) => errors.append(&mut schema_errors),
                    }
                }
                (_, ValueSchema::AllOf(all_of_schema)) => {
                    match validate_all_of(
                        value,
                        toml_version,
                        accessors,
                        all_of_schema,
                        &schema_url,
                        &definitions,
                        sub_schema_url_map,
                        schema_store,
                    )
                    .await
                    {
                        Ok(()) => {
                            valid_count += 1;
                            break;
                        }
                        Err(mut schema_errors) => errors.append(&mut schema_errors),
                    }
                }
            }
        }

        if valid_count == 1 {
            return Ok(());
        }

        if !is_type_match {
            errors.append(&mut type_mismatch_errors);
        }

        Err(errors)
    }
    .boxed()
}
