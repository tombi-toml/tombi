use std::{borrow::Cow, sync::Arc};

use tombi_future::{BoxFuture, Boxable};
use tombi_x_keyword::{ArrayValuesOrder, X_TOMBI_ARRAY_VALUES_ORDER};

use super::{
    CurrentSchema, FindSchemaCandidates, Referable, SchemaDefinitions, SchemaItem, SchemaUrl,
    ValueSchema,
};
use crate::{Accessor, SchemaStore};

#[derive(Debug, Default, Clone)]
pub struct ArraySchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub items: Option<SchemaItem>,
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
    pub unique_items: Option<bool>,
    pub enumerate: Option<Vec<tombi_json::Value>>,
    pub default: Option<tombi_json::Value>,
    pub const_value: Option<tombi_json::Value>,
    pub examples: Option<Vec<tombi_json::Value>>,
    pub values_order: Option<ArrayValuesOrder>,
    pub deprecated: Option<bool>,
}

impl ArraySchema {
    pub fn new(object: &tombi_json::ObjectNode) -> Self {
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
                    .and_then(Referable::<ValueSchema>::new)
                    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
            }),
            min_items: object
                .get("minItems")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            max_items: object
                .get("maxItems")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            unique_items: object.get("uniqueItems").and_then(|v| v.as_bool()),
            enumerate: object
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
                .and_then(|order| match order {
                    tombi_json::ValueNode::String(string) => {
                        match ArrayValuesOrder::try_from(string.value.as_ref()) {
                            Ok(val) => Some(val),
                            Err(_) => {
                                tracing::error!(
                                    "invalid {X_TOMBI_ARRAY_VALUES_ORDER}: {}",
                                    string.value
                                );
                                None
                            }
                        }
                    }
                    _ => {
                        tracing::error!(
                            "invalid {X_TOMBI_ARRAY_VALUES_ORDER}: {}",
                            order.to_string()
                        );
                        None
                    }
                }),
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
            range: object.range,
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
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)> {
        async move {
            let mut errors = Vec::new();
            let mut candidates = Vec::new();

            let Some(ref items) = self.items else {
                return (candidates, errors);
            };

            let mut referable_schema = items.write().await;
            if let Ok(Some(CurrentSchema {
                schema_url,
                value_schema,
                definitions,
            })) = referable_schema
                .resolve(
                    Cow::Borrowed(schema_url),
                    Cow::Borrowed(definitions),
                    schema_store,
                )
                .await
            {
                let (mut item_candidates, mut item_errors) = value_schema
                    .find_schema_candidates(
                        &accessors[1..],
                        &schema_url,
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
