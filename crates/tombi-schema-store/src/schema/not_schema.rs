use tombi_x_keyword::{StringFormat, X_NOT_ERROR_MESSAGE};

use crate::{AnchorCollector, DynamicAnchorCollector, SchemaItem, schema_item_from_schema_value};

#[derive(Debug, Clone)]
pub struct NotSchema {
    pub schema: SchemaItem,
    error_message: Option<String>,
}

impl NotSchema {
    #[inline]
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    #[inline]
    pub fn new(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
        dialect: Option<crate::JsonSchemaDialect>,
        anchor_collector: Option<&mut AnchorCollector>,
        dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
    ) -> Option<Self> {
        object
            .get("not")
            .and_then(|value| {
                schema_item_from_schema_value(
                    value,
                    string_formats,
                    dialect,
                    anchor_collector,
                    dynamic_anchor_collector,
                )
            })
            .map(|schema| Self {
                schema,
                error_message: object
                    .get(X_NOT_ERROR_MESSAGE)
                    .and_then(|value| value.as_str().map(str::to_string)),
            })
    }
}
