use crate::{value_type::HasValueType, ValueType};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct IntegerSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
    pub exclusive_minimum: Option<i64>,
    pub exclusive_maximum: Option<i64>,
    pub multiple_of: Option<i64>,
    pub default: Option<i64>,
}

impl IntegerSchema {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Self {
        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            minimum: object.get("minimum").and_then(|v| v.as_i64()),
            maximum: object.get("maximum").and_then(|v| v.as_i64()),
            exclusive_minimum: object.get("exclusiveMinimum").and_then(|v| v.as_i64()),
            exclusive_maximum: object.get("exclusiveMaximum").and_then(|v| v.as_i64()),
            multiple_of: object.get("multipleOf").and_then(|v| v.as_i64()),
            default: object.get("default").and_then(|v| v.as_i64()),
        }
    }
}

impl HasValueType for IntegerSchema {
    fn value_type(&self) -> ValueType {
        ValueType::Integer
    }
}
