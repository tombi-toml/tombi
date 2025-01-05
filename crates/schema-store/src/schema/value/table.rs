use std::collections::HashMap;

use crate::{value_type::HasValueType, Accessor, ValueType};

use super::ValueSchema;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TableSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub properties: HashMap<Accessor, ValueSchema>,
    pub required: Option<Vec<String>>,
    pub default: Option<serde_json::Value>,
}

impl TableSchema {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Self {
        let mut properties = HashMap::new();
        if let Some(serde_json::Value::Object(props)) = object.get("properties") {
            for (key, value) in props {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = ValueSchema::new(&object) {
                    properties.insert(Accessor::Key(key.clone()), value_schema);
                }
            }
        }

        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            properties,
            required: object.get("required").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
            }),
            default: object.get("default").cloned(),
        }
    }
}

impl HasValueType for TableSchema {
    fn value_type(&self) -> ValueType {
        ValueType::Table
    }
}