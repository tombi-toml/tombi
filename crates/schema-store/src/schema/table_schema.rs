use std::{borrow::Cow, sync::Arc};

use ahash::AHashMap;
use futures::{future::BoxFuture, FutureExt};

use super::{
    CurrentSchema, FindSchemaCandidates, SchemaDefinitions, SchemaItemTokio,
    SchemaPatternProperties, SchemaUrl, ValueSchema,
};
use crate::{Accessor, Referable, SchemaProperties, SchemaStore};

#[derive(Debug, Default, Clone)]
pub struct TableSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub properties: SchemaProperties,
    pub pattern_properties: Option<SchemaPatternProperties>,
    pub additional_properties: bool,
    pub additional_property_schema: Option<SchemaItemTokio>,
    pub required: Option<Vec<String>>,
    pub min_properties: Option<usize>,
    pub max_properties: Option<usize>,
    pub keys_order_by: Option<TableKeysOrderBy>,
}

#[derive(Debug, Clone)]
pub enum TableKeysOrderBy {
    Ascending,
    Descending,
    Schema,
}

impl TableSchema {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Self {
        let mut properties = AHashMap::new();
        if let Some(serde_json::Value::Object(props)) = object.get("properties") {
            for (key, value) in props {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = Referable::<ValueSchema>::new(object) {
                    properties.insert(Accessor::Key(key.into()), value_schema);
                }
            }
        }
        let pattern_properties = match object.get("patternProperties") {
            Some(serde_json::Value::Object(props)) => {
                let mut pattern_properties = AHashMap::new();
                for (pattern, value) in props {
                    let Some(object) = value.as_object() else {
                        continue;
                    };
                    if let Some(value_schema) = Referable::<ValueSchema>::new(object) {
                        pattern_properties.insert(pattern.clone(), value_schema);
                    }
                }
                Some(pattern_properties)
            }
            _ => None,
        };
        let (additional_properties, additional_property_schema) =
            match object.get("additionalProperties") {
                Some(serde_json::Value::Bool(allow)) => (*allow, None),
                Some(serde_json::Value::Object(object)) => {
                    let value_schema = Referable::<ValueSchema>::new(object);
                    (
                        true,
                        value_schema.map(|schema| Arc::new(tokio::sync::RwLock::new(schema))),
                    )
                }
                _ => (true, None),
            };

        let key_order = match object.get("x-tombi-table-keys-order-by") {
            Some(serde_json::Value::String(order)) => match order.as_str() {
                "ascending" => Some(TableKeysOrderBy::Ascending),
                "descending" => Some(TableKeysOrderBy::Descending),
                "schema" => Some(TableKeysOrderBy::Schema),
                _ => {
                    tracing::error!("invalid x-tombi-table-keys-order-by: {order}");
                    None
                }
            },
            Some(order) => {
                tracing::error!("invalid x-tombi-table-keys-order-by: {}", order.to_string());
                None
            }
            None => None,
        };

        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            properties: Arc::new(properties.into()),
            pattern_properties: pattern_properties.map(|props| Arc::new(props.into())),
            additional_properties,
            additional_property_schema,
            required: object.get("required").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
            }),
            min_properties: object
                .get("minProperties")
                .and_then(|v| v.as_u64().map(|u| u as usize)),
            max_properties: object
                .get("maxProperties")
                .and_then(|v| v.as_u64().map(|u| u as usize)),
            keys_order_by: key_order,
        }
    }

    pub fn value_type(&self) -> crate::ValueType {
        crate::ValueType::Table
    }

    pub fn has_additional_property_schema(&self) -> bool {
        self.additional_property_schema.is_some()
    }
}

impl FindSchemaCandidates for TableSchema {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)> {
        async move {
            let mut candidates = Vec::new();
            let mut errors = Vec::new();

            if accessors.is_empty() {
                for property in self.properties.write().await.values_mut() {
                    if let Ok(CurrentSchema {
                        value_schema,
                        schema_url,
                        definitions,
                    }) = property
                        .resolve(
                            Cow::Borrowed(schema_url),
                            Cow::Borrowed(definitions),
                            schema_store,
                        )
                        .await
                    {
                        let (schema_candidates, schema_errors) = value_schema
                            .find_schema_candidates(
                                accessors,
                                &schema_url,
                                &definitions,
                                schema_store,
                            )
                            .await;
                        candidates.extend(schema_candidates);
                        errors.extend(schema_errors);
                    }
                }

                return (candidates, errors);
            }

            if let Some(value) = self.properties.write().await.get_mut(&accessors[0]) {
                if let Ok(CurrentSchema {
                    value_schema,
                    schema_url,
                    definitions,
                }) = value
                    .resolve(
                        Cow::Borrowed(schema_url),
                        Cow::Borrowed(definitions),
                        schema_store,
                    )
                    .await
                {
                    return value_schema
                        .find_schema_candidates(
                            &accessors[1..],
                            &schema_url,
                            &definitions,
                            schema_store,
                        )
                        .await;
                }
            }

            (candidates, errors)
        }
        .boxed()
    }
}
