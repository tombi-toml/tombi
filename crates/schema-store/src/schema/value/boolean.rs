use crate::value_type::HasValueType;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BooleanSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub default: Option<bool>,
}

impl BooleanSchema {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Self {
        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            default: object.get("default").and_then(|v| v.as_bool()),
        }
    }
}

impl HasValueType for BooleanSchema {
    fn value_type(&self) -> crate::ValueType {
        crate::ValueType::Boolean
    }
}