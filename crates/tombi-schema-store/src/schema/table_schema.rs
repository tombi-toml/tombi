use std::{borrow::Cow, sync::Arc};

use ahash::AHashMap;
use indexmap::IndexMap;
use tombi_future::{BoxFuture, Boxable};
use tombi_json::StringNode;
use tombi_x_keyword::{TableKeysOrder, X_TOMBI_TABLE_KEYS_ORDER};

use super::{
    CurrentSchema, FindSchemaCandidates, PropertySchema, SchemaAccessor, SchemaDefinitions,
    SchemaItem, SchemaPatternProperties, SchemaUrl, ValueSchema,
};
use crate::{Accessor, Referable, SchemaProperties, SchemaStore};

#[derive(Debug, Default, Clone)]
pub struct TableSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub properties: SchemaProperties,
    pub pattern_properties: Option<SchemaPatternProperties>,
    additional_properties: Option<bool>,
    pub additional_property_schema: Option<(tombi_text::Range, SchemaItem)>,
    pub required: Option<Vec<String>>,
    pub min_properties: Option<usize>,
    pub max_properties: Option<usize>,
    pub keys_order: Option<TableKeysOrder>,
    pub default: Option<tombi_json::Object>,
    pub const_value: Option<tombi_json::Object>,
    pub enumerate: Option<Vec<tombi_json::Object>>,
    pub examples: Option<Vec<tombi_json::Object>>,
    pub deprecated: Option<bool>,
}

impl TableSchema {
    pub fn new(object_node: &tombi_json::ObjectNode) -> Self {
        let mut properties = IndexMap::new();
        if let Some(tombi_json::ValueNode::Object(object_node)) = object_node.get("properties") {
            for (key_node, value_node) in object_node.properties.iter() {
                let Some(object) = value_node.as_object() else {
                    continue;
                };
                if let Some(property_schema) = Referable::<ValueSchema>::new(object) {
                    properties.insert(
                        SchemaAccessor::Key(key_node.value.to_string()),
                        PropertySchema {
                            property_schema,
                            key_range: key_node.range,
                        },
                    );
                }
            }
        }
        let pattern_properties = match object_node.get("patternProperties") {
            Some(tombi_json::ValueNode::Object(object_node)) => {
                let mut pattern_properties = AHashMap::new();
                for (pattern, value) in object_node.properties.iter() {
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
            match object_node.get("additionalProperties") {
                Some(tombi_json::ValueNode::Bool(allow)) => (Some(allow.value), None),
                Some(tombi_json::ValueNode::Object(object_node)) => {
                    let value_schema = Referable::<ValueSchema>::new(object_node);
                    (
                        Some(true),
                        value_schema.map(|schema| {
                            (
                                object_node.range,
                                Arc::new(tokio::sync::RwLock::new(schema)),
                            )
                        }),
                    )
                }
                _ => (None, None),
            };

        let keys_order = match object_node.get(X_TOMBI_TABLE_KEYS_ORDER) {
            Some(tombi_json::ValueNode::String(StringNode { value: order, .. })) => {
                match TableKeysOrder::try_from(order.as_str()) {
                    Ok(val) => Some(val),
                    Err(_) => {
                        tracing::error!("invalid {X_TOMBI_TABLE_KEYS_ORDER}: {order}");
                        None
                    }
                }
            }
            Some(order) => {
                tracing::error!("invalid {X_TOMBI_TABLE_KEYS_ORDER}: {}", order.to_string());
                None
            }
            None => None,
        };

        Self {
            title: object_node
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object_node
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            range: object_node.range,
            properties: Arc::new(properties.into()),
            pattern_properties: pattern_properties.map(|props| {
                Arc::new(
                    props
                        .into_iter()
                        .map(|(key, property_schema)| {
                            (
                                key.value,
                                PropertySchema {
                                    property_schema,
                                    key_range: key.range,
                                },
                            )
                        })
                        .collect::<AHashMap<_, _>>()
                        .into(),
                )
            }),
            additional_properties,
            additional_property_schema,
            required: object_node.get("required").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.items
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(ToString::to_string)
                        .collect()
                })
            }),
            min_properties: object_node
                .get("minProperties")
                .and_then(|v| v.as_u64().map(|u| u as usize)),
            max_properties: object_node
                .get("maxProperties")
                .and_then(|v| v.as_u64().map(|u| u as usize)),
            keys_order,
            enumerate: object_node.get("enum").and_then(|v| v.as_array()).map(|v| {
                v.items
                    .iter()
                    .filter_map(|v| v.as_object().map(|v| v.into()))
                    .collect()
            }),
            default: object_node
                .get("default")
                .and_then(|v| v.as_object())
                .map(|v| v.into()),
            const_value: object_node
                .get("const")
                .and_then(|v| v.as_object())
                .map(|v| v.into()),
            examples: object_node
                .get("examples")
                .and_then(|v| v.as_array())
                .map(|v| {
                    v.items
                        .iter()
                        .filter_map(|v| v.as_object().map(|v| v.into()))
                        .collect()
                }),
            deprecated: object_node.get("deprecated").and_then(|v| v.as_bool()),
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
                for PropertySchema {
                    property_schema, ..
                } in self.properties.write().await.values_mut()
                {
                    if let Ok(Some(CurrentSchema {
                        value_schema,
                        schema_url,
                        definitions,
                    })) = property_schema
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

            if let Some(PropertySchema {
                property_schema, ..
            }) = self
                .properties
                .write()
                .await
                .get_mut(&SchemaAccessor::from(&accessors[0]))
            {
                if let Ok(Some(CurrentSchema {
                    value_schema,
                    schema_url,
                    definitions,
                })) = property_schema
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
