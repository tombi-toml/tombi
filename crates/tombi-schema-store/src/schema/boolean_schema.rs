#[derive(Debug, Default, Clone, PartialEq)]
pub struct BooleanSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub default: Option<bool>,
    pub enumerate: Option<Vec<bool>>,
    pub deprecated: Option<bool>,
}

impl BooleanSchema {
    pub fn new(object: &tombi_json_value::Map<String, tombi_json_value::Value>) -> Self {
        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            default: object.get("default").and_then(|v| v.as_bool()),
            enumerate: object
                .get("enum")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_bool()).collect()),
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
        }
    }

    pub const fn value_type(&self) -> crate::ValueType {
        crate::ValueType::Boolean
    }
}
