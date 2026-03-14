use super::{AllOfSchema, AnyOfSchema, NotSchema, OneOfSchema};

#[derive(Debug, Default, Clone)]
pub struct OffsetDateTimeSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub r#enum: Option<Vec<String>>,
    pub default: Option<String>,
    pub const_value: Option<String>,
    pub examples: Option<Vec<String>>,
    pub deprecated: Option<bool>,
    pub one_of: Option<Box<OneOfSchema>>,
    pub any_of: Option<Box<AnyOfSchema>>,
    pub all_of: Option<Box<AllOfSchema>>,
    pub not: Option<Box<NotSchema>>,
}

impl OffsetDateTimeSchema {
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
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: object
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            range: object.range,
            r#enum: object.get("enum").and_then(|v| v.as_array()).map(|a| {
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
            one_of,
            any_of,
            all_of,
            not,
        }
    }

    pub const fn value_type(&self) -> crate::ValueType {
        crate::ValueType::OffsetDateTime
    }
}
