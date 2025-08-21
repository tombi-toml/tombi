use std::str::FromStr;

use ahash::AHashMap;
use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_future::{BoxFuture, Boxable};
use tombi_x_keyword::{StringFormat, X_TOMBI_STRING_FORMATS, X_TOMBI_TOML_VERSION};

use super::{
    referable_schema::Referable, FindSchemaCandidates, SchemaDefinitions, SchemaUri, ValueSchema,
};
use crate::{Accessor, SchemaStore};

#[derive(Debug, Clone)]
pub struct DocumentSchema {
    pub schema_uri: SchemaUri,
    pub(crate) toml_version: Option<TomlVersion>,
    pub value_schema: Option<ValueSchema>,
    pub definitions: SchemaDefinitions,
}

impl DocumentSchema {
    pub fn new(object: tombi_json::ObjectNode, schema_uri: SchemaUri) -> Self {
        let toml_version = object.get(X_TOMBI_TOML_VERSION).and_then(|obj| match obj {
            tombi_json::ValueNode::String(version) => TomlVersion::from_str(&version.value).ok(),
            _ => None,
        });

        let string_formats = object
            .get(X_TOMBI_STRING_FORMATS)
            .and_then(|obj| match obj {
                tombi_json::ValueNode::Array(array) => {
                    let string_formats = array
                        .items
                        .iter()
                        .filter_map(|value| match value {
                            tombi_json::ValueNode::String(string) => {
                                StringFormat::from_str(string.value.as_str()).ok()
                            }
                            _ => None,
                        })
                        .collect_vec();
                    Some(string_formats)
                }
                _ => None,
            });

        let value_schema = ValueSchema::new(&object, string_formats.as_deref());
        let mut definitions = AHashMap::default();
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("definitions") {
            for (key, value) in object.properties.iter() {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) =
                    Referable::<ValueSchema>::new(object, string_formats.as_deref())
                {
                    definitions.insert(format!("#/definitions/{}", key.value), value_schema);
                }
            }
        }
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("$defs") {
            for (key, value) in object.properties.iter() {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) =
                    Referable::<ValueSchema>::new(object, string_formats.as_deref())
                {
                    definitions.insert(format!("#/$defs/{}", key.value), value_schema);
                }
            }
        }

        Self {
            schema_uri,
            toml_version,
            value_schema,
            definitions: SchemaDefinitions::new(definitions.into()),
        }
    }

    pub fn toml_version(&self) -> Option<TomlVersion> {
        self.toml_version.inspect(|version| {
            tracing::trace!(
                "use schema TOML version \"{version}\" for {}",
                self.schema_uri
            );
        })
    }
}

impl FindSchemaCandidates for DocumentSchema {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)> {
        async move {
            if let Some(value_schema) = &self.value_schema {
                value_schema
                    .find_schema_candidates(accessors, schema_uri, definitions, schema_store)
                    .await
            } else {
                (Vec::with_capacity(0), Vec::with_capacity(0))
            }
        }
        .boxed()
    }
}
