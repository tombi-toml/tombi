use super::{AllOfSchema, AnyOfSchema, NotSchema, OneOfSchema};

#[derive(Debug, Default, Clone)]
pub struct BooleanSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub default: Option<bool>,
    pub const_value: Option<bool>,
    pub r#enum: Option<Vec<bool>>,
    pub examples: Option<Vec<bool>>,
    pub deprecated: Option<bool>,
    pub one_of: Option<Box<OneOfSchema>>,
    pub any_of: Option<Box<AnyOfSchema>>,
    pub all_of: Option<Box<AllOfSchema>>,
    pub not: Option<Box<NotSchema>>,
}

impl BooleanSchema {
    pub fn new(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[tombi_x_keyword::StringFormat]>,
        dialect: Option<crate::JsonSchemaDialect>,
        anchor_collector: Option<&mut crate::AnchorCollector>,
        dynamic_anchor_collector: Option<&mut crate::DynamicAnchorCollector>,
    ) -> Self {
        let (one_of, any_of, all_of, not) = crate::adjacent_applicators(
            object,
            string_formats,
            dialect,
            anchor_collector,
            dynamic_anchor_collector,
        );
        Self {
            title: object
                .get("title")
                .and_then(|value| value.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|value| value.as_str().map(|s| s.to_string())),
            default: object.get("default").and_then(|v| v.as_bool()),
            const_value: object.get("const").and_then(|v| v.as_bool()),
            r#enum: object
                .get("enum")
                .and_then(|value| value.as_array())
                .map(|array| array.items.iter().filter_map(|v| v.as_bool()).collect()),
            examples: object
                .get("examples")
                .and_then(|v| v.as_array())
                .map(|array| array.items.iter().filter_map(|v| v.as_bool()).collect()),
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
            one_of,
            any_of,
            all_of,
            not,
            range: object.range,
        }
    }

    pub const fn value_type(&self) -> crate::ValueType {
        crate::ValueType::Boolean
    }
}
