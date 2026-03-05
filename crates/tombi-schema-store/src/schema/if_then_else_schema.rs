use std::sync::Arc;

use tombi_x_keyword::StringFormat;

use crate::{Referable, SchemaItem, ValueSchema};

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
    ) -> Option<Self> {
        let if_schema = object
            .get("if")
            .and_then(|value| value.as_object())
            .and_then(|obj| {
                Referable::<ValueSchema>::new(obj, string_formats)
                    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
            })?;

        let then_schema = object
            .get("then")
            .and_then(|value| value.as_object())
            .and_then(|obj| {
                Referable::<ValueSchema>::new(obj, string_formats)
                    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
            });

        let else_schema = object
            .get("else")
            .and_then(|value| value.as_object())
            .and_then(|obj| {
                Referable::<ValueSchema>::new(obj, string_formats)
                    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
            });

        Some(Self {
            if_schema,
            then_schema,
            else_schema,
        })
    }
}
