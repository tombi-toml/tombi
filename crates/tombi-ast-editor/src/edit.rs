use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, OneOfSchema, PropertySchema, SchemaAccessor, ValueSchema,
};
use tombi_validator::Validate;

mod array;
mod array_of_table;
mod inline_table;
mod key_value;
mod root;
mod table;
mod value;

pub trait Edit {
    fn edit<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>>;
}

async fn get_schema<'a: 'b, 'b>(
    value: &'a tombi_document_tree::Value,
    accessors: &'a [tombi_schema_store::Accessor],
    current_schema: &'a tombi_schema_store::CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> Option<ValueSchema> {
    fn inner_get_schema<'a: 'b, 'b>(
        value: &'a tombi_document_tree::Value,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: &'a tombi_schema_store::CurrentSchema<'a>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Option<ValueSchema>> {
        async move {
            match current_schema.value_schema.as_ref() {
                ValueSchema::Table(_) | ValueSchema::Array(_) => {}
                ValueSchema::OneOf(OneOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                | ValueSchema::AllOf(AllOfSchema { schemas, .. }) => {
                    for referable_schema in schemas.write().await.iter_mut() {
                        if let Ok(Some(current_schema)) = referable_schema
                            .resolve(
                                current_schema.schema_uri.clone(),
                                current_schema.definitions.clone(),
                                schema_context.store,
                            )
                            .await
                            .inspect_err(|err| tracing::warn!("{err}"))
                        {
                            if let Some(value_schema) =
                                inner_get_schema(value, accessors, &current_schema, schema_context)
                                    .await
                            {
                                return Some(value_schema);
                            }
                        }
                    }

                    return None;
                }
                _ => return None,
            }

            if accessors.is_empty() {
                return value
                    .validate(accessors, Some(current_schema), schema_context)
                    .await
                    .ok()
                    .map(|_| current_schema.value_schema.as_ref().clone());
            }

            match &accessors[0] {
                Accessor::Key(key) => {
                    if let (
                        tombi_document_tree::Value::Table(table),
                        ValueSchema::Table(table_schema),
                    ) = (value, current_schema.value_schema.as_ref())
                    {
                        if let Some(value) = table.get(&key.to_string()) {
                            if let Some(PropertySchema {
                                property_schema, ..
                            }) = table_schema
                                .properties
                                .write()
                                .await
                                .get_mut(&SchemaAccessor::Key(key.to_string()))
                            {
                                if let Ok(Some(current_schema)) = property_schema
                                    .resolve(
                                        current_schema.schema_uri.clone(),
                                        current_schema.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                    .inspect_err(|err| tracing::warn!("{err}"))
                                {
                                    return inner_get_schema(
                                        value,
                                        &accessors[1..],
                                        &current_schema,
                                        schema_context,
                                    )
                                    .await;
                                }
                            }
                            if let Some(pattern_properties) = &table_schema.pattern_properties {
                                for (
                                    property_key,
                                    PropertySchema {
                                        property_schema, ..
                                    },
                                ) in pattern_properties.write().await.iter_mut()
                                {
                                    if let Ok(pattern) = regex::Regex::new(property_key) {
                                        if pattern.is_match(&key.to_string()) {
                                            if let Ok(Some(current_schema)) = property_schema
                                                .resolve(
                                                    current_schema.schema_uri.clone(),
                                                    current_schema.definitions.clone(),
                                                    schema_context.store,
                                                )
                                                .await
                                                .inspect_err(|err| tracing::warn!("{err}"))
                                            {
                                                return inner_get_schema(
                                                    value,
                                                    &accessors[1..],
                                                    &current_schema,
                                                    schema_context,
                                                )
                                                .await;
                                            }
                                        }
                                    } else {
                                        tracing::warn!(
                                            "Invalid regex pattern property: {}",
                                            property_key
                                        );
                                    };
                                }
                            }
                            if let Some((_, additional_properties_schema)) =
                                &table_schema.additional_property_schema
                            {
                                if let Ok(Some(current_schema)) = additional_properties_schema
                                    .write()
                                    .await
                                    .resolve(
                                        current_schema.schema_uri.clone(),
                                        current_schema.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                    .inspect_err(|err| tracing::warn!("{err}"))
                                {
                                    return inner_get_schema(
                                        value,
                                        &accessors[1..],
                                        &current_schema,
                                        schema_context,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }
                Accessor::Index(_) => {
                    if let (
                        tombi_document_tree::Value::Array(array),
                        ValueSchema::Array(array_schema),
                    ) = (value, current_schema.value_schema.as_ref())
                    {
                        // NOTE: This is fine. This function is only used for Table/ArrayOfTable or Keys of KeyValues,
                        //       so there is only one element in the array.
                        if let Some(value) = array.first() {
                            if let Some(item_schema) = &array_schema.items {
                                if let Ok(Some(current_schema)) = item_schema
                                    .write()
                                    .await
                                    .resolve(
                                        current_schema.schema_uri.clone(),
                                        current_schema.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                    .inspect_err(|err| tracing::warn!("{err}"))
                                {
                                    return inner_get_schema(
                                        value,
                                        &accessors[1..],
                                        &current_schema,
                                        schema_context,
                                    )
                                    .await;
                                }
                            }
                        } else {
                            return None;
                        }
                    }
                }
            }

            None
        }
        .boxed()
    }

    inner_get_schema(value, accessors, current_schema, schema_context).await
}
