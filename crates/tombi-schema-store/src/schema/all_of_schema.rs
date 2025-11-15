use std::sync::Arc;

use futures::future::join_all;
use itertools::Itertools;
use tombi_x_keyword::{StringFormat, TableKeysOrder, X_TOMBI_TABLE_KEYS_ORDER};

use super::{ReferableValueSchemas, ValueSchema};
use crate::Referable;

#[derive(Debug, Default, Clone)]
pub struct AllOfSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub schemas: ReferableValueSchemas,
    pub default: Option<tombi_json::Value>,
    pub examples: Option<Vec<tombi_json::Value>>,
    pub deprecated: Option<bool>,
    pub keys_order: Option<TableKeysOrder>,
}

impl AllOfSchema {
    pub fn new(object: &tombi_json::ObjectNode, string_formats: Option<&[StringFormat]>) -> Self {
        let title = object
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = object
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let schemas = object
            .get("allOf")
            .and_then(|v| v.as_array())
            .map(|array| {
                array
                    .items
                    .iter()
                    .filter_map(|value| value.as_object())
                    .filter_map(|obj| Referable::<ValueSchema>::new(obj, string_formats))
                    .collect_vec()
            })
            .unwrap_or_default();

        Self {
            title,
            description,
            schemas: Arc::new(tokio::sync::RwLock::new(schemas)),
            default: object.get("default").cloned().map(|v| v.into()),
            examples: object
                .get("examples")
                .and_then(|value| value.as_array())
                .map(|array| array.items.iter().map(|v| v.into()).collect()),
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
            range: object.range,
            keys_order: object
                .get(X_TOMBI_TABLE_KEYS_ORDER)
                .and_then(|v| v.as_str().and_then(|s| TableKeysOrder::try_from(s).ok())),
        }
    }

    pub async fn value_type(&self) -> crate::ValueType {
        crate::ValueType::AllOf(
            join_all(
                self.schemas
                    .read()
                    .await
                    .iter()
                    .map(|schema| async { schema.value_type().await }),
            )
            .await
            .into_iter()
            .collect(),
        )
    }
}
