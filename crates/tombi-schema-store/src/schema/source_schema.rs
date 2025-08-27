use ahash::AHashMap;
use itertools::Itertools;

use super::{DocumentSchema, SchemaUri};
use crate::{SchemaAccessor, SchemaAccessors};

pub type SubSchemaUriMap = AHashMap<Vec<SchemaAccessor>, SchemaUri>;

#[derive(Clone, Default)]
pub struct SourceSchema {
    pub root_schema: Option<DocumentSchema>,
    pub sub_schema_uri_map: SubSchemaUriMap,
}

impl std::fmt::Debug for SourceSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let root_schema_uri = self
            .root_schema
            .as_ref()
            .map(|schema| schema.schema_uri.to_string());
        let sub_schema_uri_map = self
            .sub_schema_uri_map
            .iter()
            .map(|(accessors, url)| {
                format!("[{:?}]: {}", SchemaAccessors::from(accessors.clone()), url)
            })
            .collect_vec()
            .join(", ");
        write!(
            f,
            "SourceSchema {{ root_schema: {root_schema_uri:?}, sub_schema_uri_map: {sub_schema_uri_map:?} }}"
        )
    }
}
