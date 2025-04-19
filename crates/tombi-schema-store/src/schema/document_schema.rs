use ahash::AHashMap;
use futures::{future::BoxFuture, FutureExt};
use tombi_config::TomlVersion;

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
    pub fn new(
        value: tombi_json_value::Map<String, tombi_json_value::Value>,
        schema_url: SchemaUrl,
    ) -> Self {
        let toml_version = value.get("x-tombi-toml-version").and_then(|obj| match obj {
            tombi_json_value::Value::String(version) => {
                serde_json::from_str(&format!("\"{version}\"")).ok()
            }
            _ => None,
        });
        let schema_id = value
            .get("$id")
            .and_then(|v| v.as_str())
            .and_then(|s| SchemaUrl::parse(s).ok());

        let value_schema = ValueSchema::new(&value);
        let mut definitions = AHashMap::default();
        if let Some(tombi_json_value::Value::Object(object)) = value.get("definitions") {
            for (key, value) in object.into_iter() {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = Referable::<ValueSchema>::new(object) {
                    definitions.insert(format!("#/definitions/{key}"), value_schema);
                }
            }
        }
        if let Some(tombi_json_value::Value::Object(object)) = value.get("$defs") {
            for (key, value) in object.into_iter() {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = Referable::<ValueSchema>::new(object) {
                    definitions.insert(format!("#/$defs/{key}"), value_schema);
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
            tracing::debug!(
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
