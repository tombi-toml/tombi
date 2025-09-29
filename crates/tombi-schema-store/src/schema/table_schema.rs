use std::{borrow::Cow, sync::Arc};

use ahash::AHashMap;
use indexmap::IndexMap;
use itertools::Itertools;
use tombi_future::{BoxFuture, Boxable};
use tombi_x_keyword::{
    ArrayValuesOrderBy, StringFormat, TableKeysOrder, TableKeysOrderGroupKind,
    X_TOMBI_ADDITIONAL_KEY_LABEL, X_TOMBI_ARRAY_VALUES_ORDER_BY, X_TOMBI_TABLE_KEYS_ORDER,
};

use super::{
    CurrentSchema, FindSchemaCandidates, PropertySchema, SchemaAccessor, SchemaDefinitions,
    SchemaItem, SchemaPatternProperties, SchemaUri, ValueSchema,
};
use crate::{Accessor, Referable, SchemaProperties, SchemaStore};

use tombi_json::StringNode;

#[derive(Debug, Default, Clone)]
pub struct TableSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub properties: SchemaProperties,
    pub pattern_properties: Option<SchemaPatternProperties>,
    additional_properties: Option<bool>,
    pub additional_property_schema: Option<(
        tombi_text::Range, // JSON Schema property name range (for GoToTypeDefinition)
        SchemaItem,
    )>,
    pub required: Option<Vec<String>>,
    pub min_properties: Option<usize>,
    pub max_properties: Option<usize>,
    pub keys_order: Option<XTombiTableKeysOrder>,
    pub array_values_order_by: Option<ArrayValuesOrderBy>,
    pub default: Option<tombi_json::Object>,
    pub const_value: Option<tombi_json::Object>,
    pub enumerate: Option<Vec<tombi_json::Object>>,
    pub examples: Option<Vec<tombi_json::Object>>,
    pub deprecated: Option<bool>,
    pub additional_key_label: Option<String>,
}

impl TableSchema {
    pub fn new(
        object_node: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
    ) -> Self {
        let mut properties = IndexMap::new();
        if let Some(tombi_json::ValueNode::Object(object_node)) = object_node.get("properties") {
            for (key_node, value_node) in object_node.properties.iter() {
                let Some(object) = value_node.as_object() else {
                    continue;
                };
                if let Some(property_schema) = Referable::<ValueSchema>::new(object, string_formats)
                {
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
                    if let Some(value_schema) =
                        Referable::<ValueSchema>::new(object, string_formats)
                    {
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
                    let value_schema = Referable::<ValueSchema>::new(object_node, string_formats);
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

        let keys_order = object_node
            .get(X_TOMBI_TABLE_KEYS_ORDER)
            .and_then(XTombiTableKeysOrder::new);

        let array_values_order_by = object_node
            .get(X_TOMBI_ARRAY_VALUES_ORDER_BY)
            .and_then(|v| {
                if let Some(v) = v.as_str() {
                    if let Ok(v) = ArrayValuesOrderBy::try_from(v) {
                        Some(v)
                    } else {
                        tracing::warn!("Invalid {X_TOMBI_ARRAY_VALUES_ORDER_BY}: {}", v);
                        None
                    }
                } else {
                    tracing::warn!("Invalid {X_TOMBI_ARRAY_VALUES_ORDER_BY}: {}", v.to_string());
                    None
                }
            });

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
            array_values_order_by,
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
            additional_key_label: object_node
                .get(X_TOMBI_ADDITIONAL_KEY_LABEL)
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        }
    }

    pub fn value_type(&self) -> crate::ValueType {
        crate::ValueType::Table
    }

    #[inline]
    pub fn additional_properties(&self) -> Option<bool> {
        self.additional_properties
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

    pub async fn accessors(&self) -> Vec<Accessor> {
        self.properties
            .read()
            .await
            .keys()
            .map(|accessor| match accessor {
                SchemaAccessor::Key(key) => Accessor::Key(key.clone()),
                SchemaAccessor::Index => unreachable!("Table keys should not be index"),
            })
            .collect_vec()
    }
}

impl FindSchemaCandidates for TableSchema {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_uri: &'a SchemaUri,
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
                        schema_uri,
                        definitions,
                    })) = property_schema
                        .resolve(
                            Cow::Borrowed(schema_uri),
                            Cow::Borrowed(definitions),
                            schema_store,
                        )
                        .await
                    {
                        let (schema_candidates, schema_errors) = value_schema
                            .find_schema_candidates(
                                accessors,
                                &schema_uri,
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
                    schema_uri,
                    definitions,
                })) = property_schema
                    .resolve(
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        schema_store,
                    )
                    .await
                {
                    return value_schema
                        .find_schema_candidates(
                            &accessors[1..],
                            &schema_uri,
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

#[derive(Debug, Clone)]
pub enum XTombiTableKeysOrder {
    All(TableKeysOrder),
    Groups(Vec<TableKeysOrderGroup>),
}

#[derive(Debug, Clone)]
pub struct TableKeysOrderGroup {
    pub target: TableKeysOrderGroupKind,
    pub order: TableKeysOrder,
}

impl XTombiTableKeysOrder {
    pub fn new(value_node: &tombi_json::ValueNode) -> Option<Self> {
        match value_node {
            tombi_json::ValueNode::String(StringNode { value: order, .. }) => {
                match TableKeysOrder::try_from(order.as_str()) {
                    Ok(val) => Some(XTombiTableKeysOrder::All(val)),
                    Err(_) => {
                        tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER}: {order}");
                        None
                    }
                }
            }
            tombi_json::ValueNode::Object(object_node) => {
                let mut sort_orders = vec![];
                for (group_name, order) in &object_node.properties {
                    let Ok(target) = TableKeysOrderGroupKind::try_from(group_name.value.as_str())
                    else {
                        tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER} group: {group_name}");
                        return None;
                    };

                    let Some(Ok(order)) = order.as_str().map(TableKeysOrder::try_from) else {
                        tracing::warn!(
                            "Invalid {X_TOMBI_TABLE_KEYS_ORDER} {group_name} group: {order}"
                        );
                        return None;
                    };

                    if order == TableKeysOrder::Schema && target != TableKeysOrderGroupKind::Keys {
                        tracing::warn!(
                            "Invalid {X_TOMBI_TABLE_KEYS_ORDER} {group_name} group: {order}"
                        );
                        return None;
                    }

                    sort_orders.push(TableKeysOrderGroup { target, order });
                }
                Some(Self::Groups(sort_orders))
            }
            order => {
                tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER}: {}", order.to_string());
                None
            }
        }
    }
}
