use std::sync::Arc;

use tombi_x_keyword::StringFormat;

use crate::{Referable, SchemaItem, ValueSchema};

#[derive(Debug, Clone)]
pub struct NotSchema {
    pub schema: SchemaItem,
}

impl NotSchema {
    #[inline]
    pub fn new(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
    ) -> Option<Self> {
        object
            .get("not")
            .and_then(|value| value.as_object())
            .and_then(|obj| {
                Referable::<ValueSchema>::new(obj, string_formats)
                    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
            })
            .map(|schema| Self { schema })
    }
}
