use std::{borrow::Cow, sync::Arc};

use ahash::AHashMap;
use futures::{future::BoxFuture, FutureExt};
use indexmap::IndexMap;
use tombi_x_keyword::{TableKeysOrder, X_TOMBI_TABLE_KEYS_ORDER};

use super::{
    CurrentSchema, FindSchemaCandidates, SchemaAccessor, SchemaDefinitions, SchemaItemTokio,
    SchemaPatternProperties, SchemaUrl, ValueSchema,
};
use crate::{Accessor, Referable, SchemaProperties, SchemaStore};

#[derive(Debug, Default, Clone)]
pub struct TableSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub properties: SchemaProperties,
    pub pattern_properties: Option<SchemaPatternProperties>,
    additional_properties: Option<bool>,
    pub additional_property_schema: Option<SchemaItemTokio>,
    pub required: Option<Vec<String>>,
    pub min_properties: Option<usize>,
    pub max_properties: Option<usize>,
    pub keys_order: Option<TableKeysOrder>,
    pub deprecated: Option<bool>,
}

impl TableSchema {
    pub fn new(object: &tombi_json_value::Map<String, tombi_json_value::Value>) -> Self {
        let mut properties = IndexMap::new();
        if let Some(tombi_json_value::Value::Object(props)) = object.get("properties") {
            for (key, value) in props {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = Referable::<ValueSchema>::new(object) {
                    properties.insert(SchemaAccessor::Key(key.into()), value_schema);
                }
            }
        }
        let pattern_properties = match object.get("patternProperties") {
            Some(tombi_json_value::Value::Object(props)) => {
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
                Some(tombi_json_value::Value::Bool(allow)) => (Some(*allow), None),
                Some(tombi_json_value::Value::Object(object)) => {
                    let value_schema = Referable::<ValueSchema>::new(object);
                    (
                        Some(true),
                        value_schema.map(|schema| Arc::new(tokio::sync::RwLock::new(schema))),
                    )
                }
                _ => (None, None),
            };

        let keys_order = match object
            .get(X_TOMBI_TABLE_KEYS_ORDER)
            // NOTE: support old name
            .or_else(|| object.get("x-tombi-table-keys-order-by"))
        {
            Some(tombi_json_value::Value::String(order)) => match order.as_str() {
                "ascending" => Some(TableKeysOrder::Ascending),
                "descending" => Some(TableKeysOrder::Descending),
                "schema" => Some(TableKeysOrder::Schema),
                _ => {
                    tracing::error!("invalid {X_TOMBI_TABLE_KEYS_ORDER}: {order}");
                    None
                }
            },
            Some(order) => {
                tracing::error!("invalid {X_TOMBI_TABLE_KEYS_ORDER}: {}", order.to_string());
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
            keys_order,
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
        }
    }

    pub fn value_type(&self) -> crate::ValueType {
        crate::ValueType::Table
    }

    #[inline]
    pub fn allows_any_additional_properties(&self, strict: bool) -> bool {
        self.allows_additional_properties(strict) || self.pattern_properties.is_some()
    }

    #[inline]
    pub fn allows_additional_properties(&self, strict: bool) -> bool {
        self.additional_properties.unwrap_or(!strict)
    }

    #[inline]
    pub fn check_strict_additional_properties_violation(&self, strict: bool) -> bool {
        strict && self.additional_properties.is_none() && self.pattern_properties.is_none()
    }

    #[inline]
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
                    if let Ok(Some(CurrentSchema {
                        value_schema,
                        schema_url,
                        definitions,
                    })) = property
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

            if let Some(value) = self
                .properties
                .write()
                .await
                .get_mut(&SchemaAccessor::from(&accessors[0]))
            {
                if let Ok(Some(CurrentSchema {
                    value_schema,
                    schema_url,
                    definitions,
                })) = value
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
