use tombi_x_keyword::StringFormat;

use crate::{AnchorCollector, DynamicAnchorCollector, SchemaItem, schema_item_from_schema_value};

#[derive(Debug, Clone)]
pub struct IfThenElseSchema {
    pub if_schema: SchemaItem,
    pub then_schema: Option<SchemaItem>,
    pub else_schema: Option<SchemaItem>,
}

impl IfThenElseSchema {
    #[inline]
    pub fn new(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
        dialect: Option<crate::JsonSchemaDialect>,
        anchor_collector: Option<&mut AnchorCollector>,
        dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
    ) -> Option<Self> {
        let mut anchor_collector = anchor_collector;
        let mut dynamic_anchor_collector = dynamic_anchor_collector;
        let if_schema = object.get("if").and_then(|value| {
            schema_item_from_schema_value(
                value,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            )
        })?;

        let then_schema = object.get("then").and_then(|value| {
            schema_item_from_schema_value(
                value,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            )
        });

        let else_schema = object.get("else").and_then(|value| {
            schema_item_from_schema_value(
                value,
                string_formats,
                dialect,
                anchor_collector,
                dynamic_anchor_collector,
            )
        });

        Some(Self {
            if_schema,
            then_schema,
            else_schema,
        })
    }
}
