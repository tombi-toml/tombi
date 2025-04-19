#[derive(Debug, Default, Clone, PartialEq)]
pub struct StringSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub enumerate: Option<Vec<String>>,
    pub default: Option<String>,
    pub deprecated: Option<bool>,
}

impl StringSchema {
    pub fn new(object: &tombi_json_value::Map<String, tombi_json_value::Value>) -> Self {
        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            min_length: object
                .get("minLength")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            max_length: object
                .get("maxLength")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            pattern: object
                .get("pattern")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            enumerate: object.get("enum").and_then(|v| v.as_array()).map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            }),
            default: object
                .get("default")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
        }
    }

    pub const fn value_type(&self) -> crate::ValueType {
        crate::ValueType::String
    }
}
