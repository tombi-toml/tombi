use crate::{value_type::HasValueType, ValueType};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LocalTimeSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub default: Option<String>,
}

impl LocalTimeSchema {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Self {
        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            default: object
                .get("default")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        }
    }
}

impl HasValueType for LocalTimeSchema {
    fn value_type(&self) -> ValueType {
        ValueType::LocalTime
    }
}