use std::{borrow::Cow, sync::Arc};

use itertools::Itertools;
use tombi_future::{BoxFuture, Boxable};
use tombi_x_keyword::{
    ArrayValuesOrder, ArrayValuesOrderGroup, StringFormat, X_TOMBI_ARRAY_VALUES_ORDER,
};

use super::{
    CurrentSchema, FindSchemaCandidates, Referable, SchemaDefinitions, SchemaItem, SchemaUri,
    ValueSchema,
};
use crate::{
    Accessor, SchemaStore,
    schema::{if_then_else_schema::IfThenElseSchema, not_schema::NotSchema},
};

#[derive(Debug, Default, Clone)]
pub struct ArraySchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub items: Option<SchemaItem>,
    pub prefix_items: Option<Vec<SchemaItem>>,
    pub additional_items: Option<bool>,
    pub additional_items_schema: Option<SchemaItem>,
    pub contains: Option<SchemaItem>,
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
    pub unique_items: Option<bool>,
    pub r#enum: Option<Vec<tombi_json::Value>>,
    pub default: Option<tombi_json::Value>,
    pub const_value: Option<tombi_json::Value>,
    pub examples: Option<Vec<tombi_json::Value>>,
    pub values_order: Option<XTombiArrayValuesOrder>,
    pub deprecated: Option<bool>,
    pub not: Option<NotSchema>,
    pub if_then_else: Option<Box<IfThenElseSchema>>,
}

impl ArraySchema {
    pub fn new(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
        dialect: Option<crate::JsonSchemaDialect>,
    ) -> Self {
        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            items: object.get("items").and_then(|value| {
                value
                    .as_object()
                    .and_then(|obj| Referable::<ValueSchema>::new(obj, string_formats, dialect))
                    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
            }),
            prefix_items: object
                .get("prefixItems")
                .or_else(|| object.get("items").filter(|v| v.as_array().is_some()))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.items
                        .iter()
                        .filter_map(|v| {
                            v.as_object()
                                .and_then(|obj| {
                                    Referable::<ValueSchema>::new(obj, string_formats, dialect)
                                })
                                .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
                        })
                        .collect_vec()
                }),
            additional_items: if dialect == Some(crate::JsonSchemaDialect::Draft2020_12) {
                // In 2020-12, `items: false` means no overflow items (like `additionalItems: false` in draft-07)
                match object.get("items") {
                    Some(tombi_json::ValueNode::Bool(b)) => Some(b.value),
                    _ => None,
                }
            } else {
                match object.get("additionalItems") {
                    Some(tombi_json::ValueNode::Bool(b)) => Some(b.value),
                    Some(tombi_json::ValueNode::Object(_)) => Some(true),
                    _ => None,
                }
            },
            additional_items_schema: if dialect == Some(crate::JsonSchemaDialect::Draft2020_12) {
                None
            } else {
                object
                    .get("additionalItems")
                    .and_then(|v| v.as_object())
                    .and_then(|obj| Referable::<ValueSchema>::new(obj, string_formats, dialect))
                    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
            },
            contains: object.get("contains").and_then(|value| {
                value
                    .as_object()
                    .and_then(|obj| Referable::<ValueSchema>::new(obj, string_formats, dialect))
                    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
            }),
            min_items: object
                .get("minItems")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            max_items: object
                .get("maxItems")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            unique_items: object.get("uniqueItems").and_then(|v| v.as_bool()),
            r#enum: object
                .get("enum")
                .and_then(|v| v.as_array())
                .map(|array| array.items.iter().map(|v| v.into()).collect()),
            default: object
                .get("default")
                .and_then(|v| v.as_array())
                .map(|array| array.into()),
            const_value: object
                .get("const")
                .and_then(|v| v.as_array())
                .map(|array| array.into()),
            examples: object
                .get("examples")
                .and_then(|v| v.as_array())
                .map(|array| array.items.iter().map(|v| v.into()).collect()),
            values_order: object
                .get(X_TOMBI_ARRAY_VALUES_ORDER)
                .and_then(XTombiArrayValuesOrder::new),
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
            range: object.range,
            not: NotSchema::new(object, string_formats, dialect),
            if_then_else: IfThenElseSchema::new(object, string_formats, dialect).map(Box::new),
        }
    }

    pub fn value_type(&self) -> crate::ValueType {
        crate::ValueType::Array
    }
}

impl FindSchemaCandidates for ArraySchema {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)> {
        async move {
            let mut errors = Vec::new();
            let mut candidates = Vec::new();

            let Some(ref items) = self.items else {
                return (candidates, errors);
            };

            if let Ok(Some(CurrentSchema {
                schema_uri,
                value_schema,
                definitions,
            })) = crate::resolve_schema_item(
                items,
                Cow::Borrowed(schema_uri),
                Cow::Borrowed(definitions),
                schema_store,
            )
            .await
            .inspect_err(|err| log::warn!("{err}"))
            {
                let (mut item_candidates, mut item_errors) = value_schema
                    .find_schema_candidates(
                        &accessors[1..],
                        &schema_uri,
                        &definitions,
                        schema_store,
                    )
                    .await;
                candidates.append(&mut item_candidates);
                errors.append(&mut item_errors);
            };

            (candidates, errors)
        }
        .boxed()
    }
}

#[derive(Debug, Clone)]
pub enum XTombiArrayValuesOrder {
    All(ArrayValuesOrder),
    Groups(ArrayValuesOrderGroup),
}

impl XTombiArrayValuesOrder {
    pub fn new(value_node: &tombi_json::ValueNode) -> Option<Self> {
        match value_node {
            tombi_json::ValueNode::String(string) => {
                match ArrayValuesOrder::try_from(string.value.as_ref()) {
                    Ok(val) => return Some(XTombiArrayValuesOrder::All(val)),
                    Err(_) => {
                        log::warn!("Invalid {X_TOMBI_ARRAY_VALUES_ORDER}: {}", string.value);
                    }
                }
            }
            tombi_json::ValueNode::Object(object_node) => {
                for (group_name, group_orders) in &object_node.properties {
                    match group_name.value.as_str() {
                        "oneOf" => {
                            if let Some(group_orders) = group_orders.as_array() {
                                let mut orders = vec![];
                                for order in &group_orders.items {
                                    match order
                                        .as_str()
                                        .and_then(|v| ArrayValuesOrder::try_from(v).ok())
                                    {
                                        Some(val) => orders.push(val),
                                        None => {
                                            log::warn!(
                                                "Invalid {X_TOMBI_ARRAY_VALUES_ORDER} {group_name} group: {}",
                                                group_orders.to_string()
                                            );
                                        }
                                    }
                                }
                                return Some(XTombiArrayValuesOrder::Groups(
                                    ArrayValuesOrderGroup::OneOf(orders),
                                ));
                            }
                        }
                        "anyOf" => {
                            if let Some(group_orders) = group_orders.as_array() {
                                let mut orders = vec![];
                                for order in &group_orders.items {
                                    match order
                                        .as_str()
                                        .and_then(|v| ArrayValuesOrder::try_from(v).ok())
                                    {
                                        Some(val) => orders.push(val),
                                        None => {
                                            log::warn!(
                                                "Invalid {X_TOMBI_ARRAY_VALUES_ORDER} {group_name} group: {}",
                                                group_orders.to_string()
                                            );
                                        }
                                    }
                                }
                                return Some(XTombiArrayValuesOrder::Groups(
                                    ArrayValuesOrderGroup::AnyOf(orders),
                                ));
                            }
                        }
                        _ => {
                            log::warn!(
                                "Invalid {X_TOMBI_ARRAY_VALUES_ORDER} group: {}",
                                group_name.value
                            );
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }
}
