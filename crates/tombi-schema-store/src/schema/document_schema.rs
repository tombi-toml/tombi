use ahash::AHashMap;
use tombi_config::TomlVersion;
use tombi_future::{BoxFuture, Boxable};

use super::{
    referable_schema::Referable, FindSchemaCandidates, SchemaDefinitions, SchemaUrl, ValueSchema,
};
use crate::{Accessor, SchemaStore};

#[derive(Debug, Clone)]
pub struct DocumentSchema {
    pub schema_url: SchemaUrl,
    pub schema_id: Option<SchemaUrl>,
    pub(crate) toml_version: Option<TomlVersion>,
    pub value_schema: Option<ValueSchema>,
    pub definitions: SchemaDefinitions,
}

impl DocumentSchema {
    pub fn new(object: tombi_json::ObjectNode, schema_url: SchemaUrl) -> Self {
        let toml_version = object
            .get("x-tombi-toml-version")
            .and_then(|obj| match obj {
                tombi_json::ValueNode::String(version) => {
                    serde_json::from_str(&format!("\"{}\"", version.value)).ok()
                }
                _ => None,
            });
        let schema_id = object
            .get("$id")
            .and_then(|v| v.as_str())
            .and_then(|s| SchemaUrl::parse(s).ok());

        let value_schema = ValueSchema::new(&object);
        let mut definitions = AHashMap::default();
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("definitions") {
            for (key, value) in object.properties.iter() {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = Referable::<ValueSchema>::new(object) {
                    definitions.insert(format!("#/definitions/{}", key.value), value_schema);
                }
            }
        }
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("$defs") {
            for (key, value) in object.properties.iter() {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = Referable::<ValueSchema>::new(object) {
                    definitions.insert(format!("#/$defs/{}", key.value), value_schema);
                }
            }
        }

        Self {
            schema_url,
            schema_id,
            toml_version,
            value_schema,
            definitions: SchemaDefinitions::new(definitions.into()),
        }
    }

    pub fn toml_version(&self) -> Option<TomlVersion> {
        self.toml_version.inspect(|version| {
            tracing::trace!(
                "use schema TOML version \"{version}\" for {}",
                self.schema_url
            );
        })
    }
}

impl FindSchemaCandidates for DocumentSchema {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)> {
        async move {
            if let Some(value_schema) = &self.value_schema {
                value_schema
                    .find_schema_candidates(accessors, schema_url, definitions, schema_store)
                    .await
            } else {
                (Vec::with_capacity(0), Vec::with_capacity(0))
            }
        }
        .boxed()
    }
}
