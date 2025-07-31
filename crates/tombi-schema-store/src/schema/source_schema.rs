use ahash::AHashMap;

use super::{DocumentSchema, SchemaUrl};
use crate::{SchemaAccessor, SchemaAccessors};

pub type SubSchemaUrlMap = AHashMap<Vec<SchemaAccessor>, SchemaUrl>;

#[derive(Clone)]
pub struct SourceSchema {
    pub root_schema: Option<DocumentSchema>,
    pub sub_schema_url_map: SubSchemaUrlMap,
}

impl std::fmt::Debug for SourceSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let root_schema_url = self
            .root_schema
            .as_ref()
            .map(|schema| schema.schema_url.to_string());
        let sub_schema_url_map = self
            .sub_schema_url_map
            .iter()
            .map(|(accessors, url)| {
                format!("[{:?}]: {}", SchemaAccessors::new(accessors.clone()), url)
            })
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "SourceSchema {{ root_schema: {root_schema_url:?}, sub_schema_url_map: {sub_schema_url_map:?} }}"
        )
    }
}
