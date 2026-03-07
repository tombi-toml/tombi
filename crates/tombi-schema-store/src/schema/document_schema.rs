use std::{str::FromStr, sync::Arc};

use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_future::{BoxFuture, Boxable};
use tombi_x_keyword::{StringFormat, X_TOMBI_STRING_FORMATS, X_TOMBI_TOML_VERSION};

use super::{
    AnchorCollector, DynamicAnchorCollector, FindSchemaCandidates, SchemaAnchors,
    SchemaDefinitions, SchemaDynamicAnchors, SchemaUri, ValueSchema, referable_schema::Referable,
};
use crate::{Accessor, JsonSchemaDialect, SchemaStore};

#[derive(Debug, Clone)]
pub struct DocumentSchema {
    pub schema_uri: SchemaUri,
    pub(crate) dialect: Option<JsonSchemaDialect>,
    pub(crate) toml_version: Option<TomlVersion>,
    pub(crate) string_formats: Option<Vec<StringFormat>>,
    pub value_schema: Option<Arc<ValueSchema>>,
    pub definitions: SchemaDefinitions,
    pub anchors: SchemaAnchors,
    pub dynamic_anchors: SchemaDynamicAnchors,
}

impl DocumentSchema {
    pub fn new(object: tombi_json::ObjectNode, schema_uri: SchemaUri) -> Self {
        let dialect = object.get("$schema").and_then(|value| match value {
            tombi_json::ValueNode::String(s) => JsonSchemaDialect::try_from(s.value.as_str()).ok(),
            _ => None,
        });

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

        let mut anchors = AnchorCollector::default();
        let mut dynamic_anchors = DynamicAnchorCollector::default();
        let value_schema = ValueSchema::new(
            &object,
            string_formats.as_deref(),
            dialect,
            Some(&mut anchors),
            Some(&mut dynamic_anchors),
        )
        .map(Arc::new);
        let mut definitions = tombi_hashmap::HashMap::default();
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("definitions") {
            for (key, value) in object.properties.iter() {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = Referable::<ValueSchema>::new(
                    object,
                    string_formats.as_deref(),
                    dialect,
                    Some(&mut anchors),
                    Some(&mut dynamic_anchors),
                ) {
                    definitions.insert(format!("#/definitions/{}", key.value), value_schema);
                }
            }
        }
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("$defs") {
            for (key, value) in object.properties.iter() {
                let Some(object) = value.as_object() else {
                    continue;
                };
                if let Some(value_schema) = Referable::<ValueSchema>::new(
                    object,
                    string_formats.as_deref(),
                    dialect,
                    Some(&mut anchors),
                    Some(&mut dynamic_anchors),
                ) {
                    definitions.insert(format!("#/$defs/{}", key.value), value_schema);
                }
            }
        }

        if let Some(value_schema) = value_schema.as_ref() {
            let root_referable = Referable::Resolved {
                schema_uri: None,
                value: value_schema.clone(),
            };
            super::collect_named_anchors(
                &object,
                &root_referable,
                Some(&mut anchors),
                Some(&mut dynamic_anchors),
            );
        }
        Self {
            schema_uri,
            dialect,
            toml_version,
            string_formats,
            value_schema,
            definitions: SchemaDefinitions::new(definitions.into()),
            anchors: SchemaAnchors::new(anchors.into()),
            dynamic_anchors: SchemaDynamicAnchors::new(dynamic_anchors.into()),
        }
    }

    pub fn dialect(&self) -> Option<JsonSchemaDialect> {
        self.dialect
    }

    pub fn string_formats(&self) -> Option<&[StringFormat]> {
        self.string_formats.as_deref()
    }

    pub fn toml_version(&self) -> Option<TomlVersion> {
        self.toml_version.inspect(|version| {
            log::trace!(
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::DocumentSchema;

    #[test]
    fn collects_anchor_definitions_for_2019_09_and_later() {
        let schema_json = r#"{
            "$schema": "https://json-schema.org/draft/2019-09/schema",
            "type": "object",
            "properties": {
                "name": {
                    "$anchor": "nameSchema",
                    "type": "string"
                }
            }
        }"#;

        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid schema json");
        let object = schema_value
            .as_object()
            .expect("schema must be object")
            .to_owned();
        let schema_uri = tombi_uri::SchemaUri::from_str("https://example.com/schema.json")
            .expect("valid schema uri");

        let document_schema = DocumentSchema::new(object, schema_uri);
        let definitions = document_schema.definitions.blocking_read();
        assert!(!definitions.contains_key("#nameSchema"));
        let anchors = document_schema.anchors.blocking_read();
        assert!(anchors.contains_key("#nameSchema"));
    }

    #[test]
    fn collects_dynamic_anchor_definitions_for_2020_12() {
        let schema_json = r#"{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "name": {
                    "$dynamicAnchor": "nameSchema",
                    "type": "string"
                }
            }
        }"#;

        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid schema json");
        let object = schema_value
            .as_object()
            .expect("schema must be object")
            .to_owned();
        let schema_uri = tombi_uri::SchemaUri::from_str("https://example.com/schema.json")
            .expect("valid schema uri");

        let document_schema = DocumentSchema::new(object, schema_uri);
        let dynamic_anchors = document_schema.dynamic_anchors.blocking_read();
        assert!(dynamic_anchors.contains_key("#nameSchema"));
    }
}
