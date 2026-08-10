use std::{borrow::Cow, str::FromStr, sync::Arc};

use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_type::BoolDefaultTrue;
use tombi_x_keyword::{StringFormat, X_TOMBI_STRING_FORMATS, X_TOMBI_TOML_VERSION};

use super::{
    AnchorCollector, CurrentSchema, DynamicAnchorCollector, FindSchemaCandidates, SchemaAnchors,
    SchemaDefinitions, SchemaDynamicAnchors, SchemaUri, SchemaView, SemanticSchema,
    referable_schema::Referable,
};
use crate::{Accessor, JsonSchemaDialect, SchemaStore};

#[derive(Debug, Clone)]
pub struct DocumentSchema {
    pub id: Option<SchemaUri>,
    pub schema_uri: SchemaUri,
    /// strict setting on root-schema level.
    pub strict: Option<BoolDefaultTrue>,
    pub(crate) dialect: Option<JsonSchemaDialect>,
    pub(crate) toml_version: Option<TomlVersion>,
    pub(crate) string_formats: Option<Vec<StringFormat>>,
    pub(crate) format_assertion: bool,
    pub schema_view: Option<Arc<SchemaView>>,
    pub semantic_schema: Option<Arc<SemanticSchema>>,
    pub definitions: SchemaDefinitions,
    pub anchors: SchemaAnchors,
    pub dynamic_anchors: SchemaDynamicAnchors,
}

impl DocumentSchema {
    pub async fn new(
        node: tombi_json::ValueNode,
        schema_uri: SchemaUri,
        strict: Option<BoolDefaultTrue>,
        schema_store: &SchemaStore,
    ) -> Self {
        match node {
            tombi_json::ValueNode::Object(object) => {
                Self::new_from_object(object, schema_uri, strict, schema_store).await
            }
            tombi_json::ValueNode::Bool(bool) => Self {
                id: None,
                schema_uri,
                strict,
                dialect: None,
                toml_version: None,
                string_formats: None,
                format_assertion: true,
                schema_view: Some(Arc::new(super::bool_schema_view(bool.value, bool.range))),
                semantic_schema: SemanticSchema::from_value_node(
                    &tombi_json::ValueNode::Bool(bool),
                    None,
                )
                .map(Arc::new),
                definitions: SchemaDefinitions::new(Default::default()),
                anchors: SchemaAnchors::new(Default::default()),
                dynamic_anchors: SchemaDynamicAnchors::new(Default::default()),
            },
            _ => Self {
                id: None,
                schema_uri,
                strict,
                dialect: None,
                toml_version: None,
                string_formats: None,
                format_assertion: true,
                schema_view: None,
                semantic_schema: None,
                definitions: SchemaDefinitions::new(Default::default()),
                anchors: SchemaAnchors::new(Default::default()),
                dynamic_anchors: SchemaDynamicAnchors::new(Default::default()),
            },
        }
    }

    async fn new_from_object(
        object: tombi_json::ObjectNode,
        schema_uri: SchemaUri,
        strict: Option<BoolDefaultTrue>,
        schema_store: &SchemaStore,
    ) -> Self {
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
        let collect_anchor = crate::supports_keyword(dialect, "$anchor");
        let collect_dynamic_anchor = crate::supports_keyword(dialect, "$dynamicAnchor")
            || crate::supports_keyword(dialect, "$recursiveAnchor");
        // The root value schema may itself be a `$ref`. A direct schema resolves to an
        // `Arc` immediately; a root `$ref` is resolved below once the definitions are built.
        let (mut schema_view, semantic_schema, root_ref) = match Referable::new(
            &object,
            string_formats.as_deref(),
            dialect,
            collect_anchor.then_some(&mut anchors),
            collect_dynamic_anchor.then_some(&mut dynamic_anchors),
        ) {
            Some(Referable::Resolved {
                value,
                semantic_schema,
                ..
            }) => (Some(value), semantic_schema, None),
            Some(root_ref @ Referable::Ref { .. }) => (
                None,
                Some(Arc::new(SemanticSchema::from_object_node(&object, dialect))),
                Some(root_ref),
            ),
            None => (None, None, None),
        };

        // A TOML document root is always an object. Select that concrete view at
        // the document boundary while keeping the semantic schema authoritative.
        if matches!(schema_view.as_deref(), Some(SchemaView::Anything(_)))
            && let Some(object_view) = semantic_schema.as_deref().and_then(|schema| {
                schema.schema_view_for_type(super::SchemaType::Object, string_formats.as_deref())
            })
        {
            schema_view = Some(Arc::new(object_view));
        }

        let mut definitions = tombi_hashmap::HashMap::default();
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("definitions") {
            for (key, value) in object.properties.iter() {
                if let Some(schema_view) = super::referable_from_schema_value(
                    value,
                    string_formats.as_deref(),
                    dialect,
                    collect_anchor.then_some(&mut anchors),
                    collect_dynamic_anchor.then_some(&mut dynamic_anchors),
                ) {
                    definitions.insert(format!("#/definitions/{}", key.value), schema_view);
                }
            }
        }
        if let Some(tombi_json::ValueNode::Object(object)) = object.get("$defs") {
            for (key, value) in object.properties.iter() {
                if let Some(schema_view) = super::referable_from_schema_value(
                    value,
                    string_formats.as_deref(),
                    dialect,
                    collect_anchor.then_some(&mut anchors),
                    collect_dynamic_anchor.then_some(&mut dynamic_anchors),
                ) {
                    definitions.insert(format!("#/$defs/{}", key.value), schema_view);
                }
            }
        }

        let mut document_schema = Self {
            id,
            schema_uri,
            strict,
            dialect,
            toml_version,
            string_formats,
            format_assertion,
            schema_view,
            semantic_schema,
            definitions: SchemaDefinitions::new(definitions.into()),
            anchors: SchemaAnchors::new(anchors.into()),
            dynamic_anchors: SchemaDynamicAnchors::new(dynamic_anchors.into()),
        };

        // Resolve a root-level `$ref` once at load time so the document exposes a usable
        // value schema (e.g. schemas whose root is only `{ "$ref": "#/definitions/..." }`).
        // `definitions` / `base_uri` are borrowed only until the resolved value is built.
        if let Some(mut root_ref) = root_ref {
            document_schema.schema_view = match root_ref
                .resolve(
                    Cow::Owned(document_schema.base_uri().clone()),
                    Cow::Owned(document_schema.definitions.clone()),
                    strict,
                    schema_store,
                )
                .await
            {
                Ok(resolved) => {
                    if let Some(current_schema) = resolved {
                        document_schema.semantic_schema = current_schema.semantic_schema;
                        Some(current_schema.schema_view)
                    } else {
                        None
                    }
                }
                Err(error) => {
                    log::warn!(
                        "failed to resolve root $ref for {}: {error}",
                        document_schema.schema_uri
                    );
                    None
                }
            };
        }

        document_schema
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

    pub fn as_current_schema(&self) -> Option<CurrentSchema<'_>> {
        self.schema_view.as_ref().map(|schema_view| CurrentSchema {
            schema_view: schema_view.clone(),
            semantic_schema: self.semantic_schema.clone(),
            schema_uri: Cow::Borrowed(&self.schema_uri),
            definitions: Cow::Borrowed(&self.definitions),
            strict: self.strict,
        })
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
        strict: Option<BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<SchemaView>, Vec<crate::Error>)> {
        async move {
            if let Some(schema_view) = &self.schema_view {
                schema_view
                    .find_schema_candidates(
                        accessors,
                        schema_uri,
                        definitions,
                        strict,
                        schema_store,
                    )
                    .await
            } else {
                (Vec::new(), Vec::new())
            }
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{SchemaStore, SchemaView};

    use super::DocumentSchema;

    #[tokio::test]
    async fn collects_anchor_definitions_for_2019_09_and_later() {
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

        let document_schema =
            DocumentSchema::new(schema_value, schema_uri, None, &SchemaStore::new()).await;
        let definitions = document_schema.definitions.read().await;
        assert!(!definitions.contains_key("#nameSchema"));
        let anchors = document_schema.anchors.read().await;
        assert!(anchors.contains_key("#nameSchema"));
    }

    #[tokio::test]
    async fn format_assertion_default_true_for_draft_07() {
        let schema_json = r#"{ "$schema": "http://json-schema.org/draft-07/schema#" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        assert!(doc.format_assertion());
    }

    #[tokio::test]
    async fn format_assertion_default_false_for_2019_09() {
        let schema_json = r#"{ "$schema": "https://json-schema.org/draft/2019-09/schema" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        assert!(!doc.format_assertion());
    }

    #[tokio::test]
    async fn format_assertion_enabled_by_2019_09_vocabulary() {
        let schema_json = r#"{
            "$schema": "https://json-schema.org/draft/2019-09/schema",
            "$vocabulary": {
                "https://json-schema.org/draft/2019-09/vocab/format": true
            }
        }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        assert!(doc.format_assertion());
    }

    #[tokio::test]
    async fn format_assertion_disabled_by_2019_09_vocabulary_false() {
        let schema_json = r#"{
            "$schema": "https://json-schema.org/draft/2019-09/schema",
            "$vocabulary": {
                "https://json-schema.org/draft/2019-09/vocab/format": false
            }
        }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        assert!(!doc.format_assertion());
    }

    #[tokio::test]
    async fn format_assertion_default_false_for_2020_12() {
        let schema_json = r#"{ "$schema": "https://json-schema.org/draft/2020-12/schema" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        assert!(!doc.format_assertion());
    }

    #[tokio::test]
    async fn format_assertion_enabled_by_vocabulary() {
        let schema_json = r#"{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$vocabulary": {
                "https://json-schema.org/draft/2020-12/vocab/format-assertion": true
            }
        }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        assert!(doc.format_assertion());
    }

    #[tokio::test]
    async fn collects_dynamic_anchor_definitions_for_2020_12() {
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

        let document_schema =
            DocumentSchema::new(schema_value, schema_uri, None, &SchemaStore::new()).await;
        let dynamic_anchors = document_schema.dynamic_anchors.read().await;
        assert!(dynamic_anchors.contains_key("#nameSchema"));
    }

    #[tokio::test]
    async fn root_boolean_true_schema_is_accepted() {
        let schema_value = tombi_json::ValueNode::from_str("true").expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        std::assert_matches!(doc.schema_view.as_deref(), Some(SchemaView::Anything(_)));
    }

    #[tokio::test]
    async fn root_boolean_false_schema_is_accepted() {
        let schema_value = tombi_json::ValueNode::from_str("false").expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/s.json").expect("valid uri");
        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        std::assert_matches!(doc.schema_view.as_deref(), Some(SchemaView::Nothing(_)));
    }

    #[tokio::test]
    async fn base_uri_uses_absolute_id_when_present() {
        let schema_json = r#"{ "$id": "https://example.com/other/schema.json" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/base/root.json")
            .expect("valid uri");

        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        let expected = tombi_uri::SchemaUri::from_str("https://example.com/other/schema.json")
            .expect("valid uri");
        assert_eq!(doc.id.as_ref(), Some(&expected));
        assert_eq!(doc.base_uri(), &expected);
    }

    #[tokio::test]
    async fn base_uri_uses_resolved_relative_id_when_present() {
        let schema_json = r#"{ "$id": "defs/schema.json" }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/base/root.json")
            .expect("valid uri");

        let doc = DocumentSchema::new(schema_value, uri, None, &SchemaStore::new()).await;
        let expected = tombi_uri::SchemaUri::from_str("https://example.com/base/defs/schema.json")
            .expect("valid uri");
        assert_eq!(doc.id.as_ref(), Some(&expected));
        assert_eq!(doc.base_uri(), &expected);
    }

    #[tokio::test]
    async fn base_uri_falls_back_to_schema_uri_when_id_is_not_string() {
        let schema_json = r#"{ "$id": 1 }"#;
        let schema_value = tombi_json::ValueNode::from_str(schema_json).expect("valid");
        let uri = tombi_uri::SchemaUri::from_str("https://example.com/base/root.json")
            .expect("valid uri");

        let doc = DocumentSchema::new(schema_value, uri.clone(), None, &SchemaStore::new()).await;
        assert_eq!(doc.id, None);
        assert_eq!(doc.base_uri(), &uri);
    }
}
