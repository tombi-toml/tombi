use tombi_x_keyword::StringFormat;

use super::{AllOfSchema, AnyOfSchema, NotSchema, OneOfSchema};

#[derive(Debug, Default, Clone)]
pub struct StringSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
    pub content_encoding: Option<String>,
    pub content_media_type: Option<String>,
    pub content_schema: Option<tombi_json::Value>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub format: Option<StringFormat>,
    pub pattern: Option<String>,
    pub r#enum: Option<Vec<String>>,
    pub examples: Option<Vec<String>>,
    pub default: Option<String>,
    pub const_value: Option<String>,
    pub deprecated: Option<bool>,
    pub one_of: Option<Box<OneOfSchema>>,
    pub any_of: Option<Box<AnyOfSchema>>,
    pub all_of: Option<Box<AllOfSchema>>,
    pub not: Option<Box<NotSchema>>,
}

impl StringSchema {
    pub fn new(
        object: &tombi_json::ObjectNode,
        format: Option<StringFormat>,
        string_formats: Option<&[StringFormat]>,
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
            content_encoding: object
                .get("contentEncoding")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            content_media_type: object
                .get("contentMediaType")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            content_schema: object.get("contentSchema").cloned().map(Into::into),
            min_length: object
                .get("minLength")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            max_length: object
                .get("maxLength")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            format,
            pattern: object
                .get("pattern")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            r#enum: object.get("enum").and_then(|v| v.as_array()).map(|a| {
                a.items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect()
            }),
            const_value: object
                .get("const")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            examples: object.get("examples").and_then(|v| v.as_array()).map(|a| {
                a.items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect()
            }),
            default: object
                .get("default")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
            one_of,
            any_of,
            all_of,
            not,
        }
    }

    pub const fn value_type(&self) -> crate::ValueType {
        crate::ValueType::String
    }
}
