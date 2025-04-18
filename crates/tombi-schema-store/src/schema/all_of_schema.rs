use std::sync::Arc;

use futures::future::join_all;

use super::{ReferableValueSchemas, ValueSchema};
use crate::Referable;

#[derive(Debug, Default, Clone)]
pub struct AllOfSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub schemas: ReferableValueSchemas,
    pub default: Option<serde_json::Value>,
    pub deprecated: Option<bool>,
}

impl AllOfSchema {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Self {
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
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_object())
                    .filter_map(Referable::<ValueSchema>::new)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let default = object.get("default").cloned();

        Self {
            title,
            description,
            schemas: Arc::new(tokio::sync::RwLock::new(schemas)),
            default,
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
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
