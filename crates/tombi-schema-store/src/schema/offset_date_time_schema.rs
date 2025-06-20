#[derive(Debug, Default, Clone, PartialEq)]
pub struct OffsetDateTimeSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub enumerate: Option<Vec<String>>,
    pub default: Option<String>,
    pub const_value: Option<String>,
    pub examples: Option<Vec<String>>,
    pub deprecated: Option<bool>,
}

impl OffsetDateTimeSchema {
    pub fn new(object: &tombi_json::ObjectNode) -> Self {
        Self {
            title: object
                .get("title")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            range: object.range,
            enumerate: object.get("enum").and_then(|v| v.as_array()).map(|a| {
                a.items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect()
            }),
            default: object
                .get("default")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            const_value: object
                .get("const")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            examples: object.get("examples").and_then(|v| v.as_array()).map(|v| {
                v.items
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect()
            }),
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
        }
    }

    pub const fn value_type(&self) -> crate::ValueType {
        crate::ValueType::OffsetDateTime
    }
}
