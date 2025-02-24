use std::sync::Arc;

use super::FindSchemaCandidates;
use super::SchemaDefinitions;
use super::SchemaItemTokio;
use super::SchemaPatternProperties;
use super::ValueSchema;
use crate::SchemaStore;
use crate::{Accessor, Referable, SchemaProperties};
use ahash::AHashMap;
use futures::future::BoxFuture;
use futures::FutureExt;

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
    pub key_order: Option<TableKeyOrder>,
}

#[derive(Debug, Clone)]
pub enum TableKeyOrder {
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

        let key_order = match object.get("x-tombi-table-key-order-by") {
            Some(serde_json::Value::String(order)) => match order.as_str() {
                "ascending" => Some(TableKeyOrder::Ascending),
                "descending" => Some(TableKeyOrder::Descending),
                "schema" => Some(TableKeyOrder::Schema),
                _ => {
                    tracing::error!("invalid x-tombi-table-key-order-by: {order}");
                    None
                }
            },
            Some(order) => {
                tracing::error!("invalid x-tombi-table-key-order-by: {}", order.to_string());
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
            key_order,
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
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)> {
        async move {
            let mut candidates = Vec::new();
            let mut errors = Vec::new();

            if accessors.is_empty() {
                for property in self.properties.write().await.values_mut() {
                    if let Ok((value_schema, new_schema)) =
                        property.resolve(definitions, schema_store).await
                    {
                        let definitions = if let Some((_, definitions)) = &new_schema {
                            definitions
                        } else {
                            definitions
                        };
                        let (schema_candidates, schema_errors) = value_schema
                            .find_schema_candidates(accessors, definitions, schema_store)
                            .await;
                        candidates.extend(schema_candidates);
                        errors.extend(schema_errors);
                    }
                }

                return (candidates, errors);
            }

            if let Some(value) = self.properties.write().await.get_mut(&accessors[0]) {
                if let Ok((value_schema, new_schema)) =
                    value.resolve(definitions, schema_store).await
                {
                    let definitions = if let Some((_, definitions)) = &new_schema {
                        definitions
                    } else {
                        definitions
                    };

                    return value_schema
                        .find_schema_candidates(&accessors[1..], definitions, schema_store)
                        .await;
                }
            }

            (candidates, errors)
        }
        .boxed()
    }
}
