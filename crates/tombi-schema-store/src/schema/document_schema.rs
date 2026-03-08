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
    pub id: Option<SchemaUri>,
    pub schema_uri: SchemaUri,
    pub(crate) dialect: Option<JsonSchemaDialect>,
    pub(crate) toml_version: Option<TomlVersion>,
    pub(crate) string_formats: Option<Vec<StringFormat>>,
    pub(crate) format_assertion: bool,
    pub value_schema: Option<Arc<ValueSchema>>,
    pub definitions: SchemaDefinitions,
    pub anchors: SchemaAnchors,
    pub dynamic_anchors: SchemaDynamicAnchors,
}

impl DocumentSchema {
    pub fn new(node: tombi_json::ValueNode, schema_uri: SchemaUri) -> Self {
        match node {
            tombi_json::ValueNode::Object(object) => Self::new_from_object(object, schema_uri),
            tombi_json::ValueNode::Bool(boolean_schema) => Self {
                id: None,
                schema_uri,
                dialect: None,
                toml_version: None,
                string_formats: None,
                format_assertion: true,
                value_schema: Some(Arc::new(super::boolean_value_schema(boolean_schema.value))),
                definitions: SchemaDefinitions::new(Default::default()),
                anchors: SchemaAnchors::new(Default::default()),
                dynamic_anchors: SchemaDynamicAnchors::new(Default::default()),
            },
            _ => Self {
                id: None,
                schema_uri,
                dialect: None,
                toml_version: None,
                string_formats: None,
                format_assertion: true,
                value_schema: None,
                definitions: SchemaDefinitions::new(Default::default()),
                anchors: SchemaAnchors::new(Default::default()),
                dynamic_anchors: SchemaDynamicAnchors::new(Default::default()),
            },
        }
    }

    fn new_from_object(object: tombi_json::ObjectNode, schema_uri: SchemaUri) -> Self {
        let id = resolve_schema_id(&object, &schema_uri);

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

        const FORMAT_2019_VOCAB: &str = "https://json-schema.org/draft/2019-09/vocab/format";
        const FORMAT_ASSERTION_2020_VOCAB: &str =
            "https://json-schema.org/draft/2020-12/vocab/format-assertion";
        let format_assertion = match dialect {
            Some(JsonSchemaDialect::Draft07) | None => true,
            Some(JsonSchemaDialect::Draft2019_09) => {
                has_enabled_vocabulary(&object, FORMAT_2019_VOCAB)
            }
            Some(JsonSchemaDialect::Draft2020_12) => {
                has_enabled_vocabulary(&object, FORMAT_ASSERTION_2020_VOCAB)
            }
        };

        let mut anchors = AnchorCollector::default();
        let mut dynamic_anchors = DynamicAnchorCollector::default();
        let collect_anchor =
            dialect.is_some_and(|dialect| crate::supports_keyword(dialect, "$anchor"));
        let collect_dynamic_anchor = dialect.is_some_and(|dialect| {
            crate::supports_keyword(dialect, "$dynamicAnchor")
                || crate::supports_keyword(dialect, "$recursiveAnchor")
        });
        let value_schema = ValueSchema::new(
            &object,
            string_formats.as_deref(),
            dialect,
            collect_anchor.then_some(&mut anchors),
            collect_dynamic_anchor.then_some(&mut dynamic_anchors),
        )
        .map(Arc::new);

        let mut definitions = tombi_hashmap::HashMap::default();
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("definitions") {
            for (key, value) in object.properties.iter() {
                if let Some(value_schema) = super::referable_from_schema_value(
                    value,
                    string_formats.as_deref(),
                    dialect,
                    collect_anchor.then_some(&mut anchors),
                    collect_dynamic_anchor.then_some(&mut dynamic_anchors),
                ) {
                    definitions.insert(format!("#/definitions/{}", key.value), value_schema);
                }
            }
        }
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("$defs") {
            for (key, value) in object.properties.iter() {
                if let Some(value_schema) = super::referable_from_schema_value(
                    value,
                    string_formats.as_deref(),
                    dialect,
                    collect_anchor.then_some(&mut anchors),
                    collect_dynamic_anchor.then_some(&mut dynamic_anchors),
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
            super::update_named_anchors(
                &object,
                &root_referable,
                dialect,
                collect_anchor.then_some(&mut anchors),
                collect_dynamic_anchor.then_some(&mut dynamic_anchors),
            );
        }
        Self {
            id,
            schema_uri,
            dialect,
            toml_version,
            string_formats,
            format_assertion,
            value_schema,
            definitions: SchemaDefinitions::new(definitions.into()),
            anchors: SchemaAnchors::new(anchors.into()),
            dynamic_anchors: SchemaDynamicAnchors::new(dynamic_anchors.into()),
        }
    }

    pub fn dialect(&self) -> Option<JsonSchemaDialect> {
        self.dialect
    }

    pub fn format_assertion(&self) -> bool {
        self.format_assertion
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

    pub fn base_uri(&self) -> &SchemaUri {
        self.id.as_ref().unwrap_or(&self.schema_uri)
    }
}

fn has_enabled_vocabulary(object: &tombi_json::ObjectNode, vocabulary_uri: &str) -> bool {
    object
        .get("$vocabulary")
        .and_then(|v| v.as_object())
        .and_then(|vocab| vocab.get(vocabulary_uri))
        .is_some_and(|value| matches!(value, tombi_json::ValueNode::Bool(b) if b.value))
}

fn resolve_schema_id(
    object: &tombi_json::ObjectNode,
    base_schema_uri: &SchemaUri,
) -> Option<SchemaUri> {
    let id = object.get("$id")?.as_str()?;
    if let Ok(joined) = base_schema_uri.join(id) {
        return Some(SchemaUri::from(joined));
    }
    SchemaUri::from_str(id).ok()
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

    use crate::ValueSchema;

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
        let schema_uri = tombi_uri::SchemaUri::from_str("https://example.com/schema.json")
            .expect("valid schema uri");

        let document_schema = DocumentSchema::new(schema_value, schema_uri);
        let definitions = document_schema.definitions.blocking_read();
        assert!(!definitions.contains_key("#nameSchema"));
        let anchors = document_schema.anchors.blocking_read();
        assert!(anchors.contains_key("#nameSchema"));
    }

    #[test]
    fn format_assertion_default_true_for_draft_07() {
        let schema_json = r#"{ "$schema": "http://json-schema.org/draft-07/schema#" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri);
        assert!(doc.format_assertion());
    }

    #[test]
    fn format_assertion_default_false_for_2019_09() {
        let schema_json = r#"{ "$schema": "https://json-schema.org/draft/2019-09/schema" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri);
        assert!(!doc.format_assertion());
    }

    #[test]
    fn format_assertion_enabled_by_2019_09_vocabulary() {
        let schema_json = r#"{
            "$schema": "https://json-schema.org/draft/2019-09/schema",
            "$vocabulary": {
                "https://json-schema.org/draft/2019-09/vocab/format": true
            }
        }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri);
        assert!(doc.format_assertion());
    }

    #[test]
    fn format_assertion_disabled_by_2019_09_vocabulary_false() {
        let schema_json = r#"{
            "$schema": "https://json-schema.org/draft/2019-09/schema",
            "$vocabulary": {
                "https://json-schema.org/draft/2019-09/vocab/format": false
            }
        }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri);
        assert!(!doc.format_assertion());
    }

    #[test]
    fn format_assertion_default_false_for_2020_12() {
        let schema_json = r#"{ "$schema": "https://json-schema.org/draft/2020-12/schema" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri);
        assert!(!doc.format_assertion());
    }

    #[test]
    fn format_assertion_enabled_by_vocabulary() {
        let schema_json = r#"{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$vocabulary": {
                "https://json-schema.org/draft/2020-12/vocab/format-assertion": true
            }
        }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri);
        assert!(doc.format_assertion());
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
        let schema_uri = tombi_uri::SchemaUri::from_str("https://example.com/schema.json")
            .expect("valid schema uri");

        let document_schema = DocumentSchema::new(schema_value, schema_uri);
        let dynamic_anchors = document_schema.dynamic_anchors.blocking_read();
        assert!(dynamic_anchors.contains_key("#nameSchema"));
    }

    #[test]
    fn root_boolean_true_schema_is_accepted() {
        let schema_value = tombi_json::ValueNode::from_str("true").expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri);
        assert!(matches!(
            doc.value_schema.as_deref(),
            Some(ValueSchema::AnyOf(any_of)) if any_of.schemas.blocking_read().is_empty()
        ));
    }

    #[test]
    fn root_boolean_false_schema_is_accepted() {
        let schema_value = tombi_json::ValueNode::from_str("false").expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri);
        assert!(matches!(
            doc.value_schema.as_deref(),
            Some(ValueSchema::OneOf(one_of)) if one_of.schemas.blocking_read().is_empty()
        ));
    }

    #[test]
    fn base_uri_uses_absolute_id_when_present() {
        let schema_json = r#"{ "$id": "https://example.com/other/schema.json" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/base/root.json")
            .expect("valid uri");

        let doc = DocumentSchema::new(schema_value, uri);
        let expected = tombi_uri::SchemaUri::from_str("https://example.com/other/schema.json")
            .expect("valid uri");
        assert_eq!(doc.id.as_ref(), Some(&expected));
        assert_eq!(doc.base_uri(), &expected);
    }

    #[test]
    fn base_uri_uses_resolved_relative_id_when_present() {
        let schema_json = r#"{ "$id": "defs/schema.json" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/base/root.json")
            .expect("valid uri");

        let doc = DocumentSchema::new(schema_value, uri);
        let expected = tombi_uri::SchemaUri::from_str("https://example.com/base/defs/schema.json")
            .expect("valid uri");
        assert_eq!(doc.id.as_ref(), Some(&expected));
        assert_eq!(doc.base_uri(), &expected);
    }

    #[test]
    fn base_uri_falls_back_to_schema_uri_when_id_is_not_string() {
        let schema_json = r#"{ "$id": 1 }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/base/root.json")
            .expect("valid uri");

        let doc = DocumentSchema::new(schema_value, uri.clone());
        assert_eq!(doc.id, None);
        assert_eq!(doc.base_uri(), &uri);
    }
}
