use std::{borrow::Cow, fmt::Debug};

use config::TomlVersion;
use futures::{future::BoxFuture, FutureExt};
use schema_store::CurrentSchema;

use super::Validate;

pub fn validate_all_of<'a: 'b, 'b, T>(
    value: &'a T,
    toml_version: TomlVersion,
    accessors: &'a [schema_store::Accessor],
    all_of_schema: &'a schema_store::AllOfSchema,
    schema_url: &'a schema_store::SchemaUrl,
    definitions: &'a schema_store::SchemaDefinitions,
    sub_schema_url_map: &'a schema_store::SubSchemaUrlMap,
    schema_store: &'a schema_store::SchemaStore,
) -> BoxFuture<'b, Result<(), Vec<crate::Error>>>
where
    T: Validate + Sync + Send + Debug,
{
    tracing::trace!("value = {:?}", value);
    tracing::trace!("all_of_schema = {:?}", all_of_schema);

    async move {
        let mut errors = vec![];

        let mut schemas = all_of_schema.schemas.write().await;
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
                Ok(()) => {}
                Err(mut schema_errors) => errors.append(&mut schema_errors),
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
