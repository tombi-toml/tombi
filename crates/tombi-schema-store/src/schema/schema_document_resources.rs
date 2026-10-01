use std::{str::FromStr, sync::Arc};

use super::SchemaUri;
use crate::{JsonSchemaDialect, SchemaStore};

#[derive(Debug, Clone)]
pub(crate) struct SchemaResource {
    /// **schema_resource_uri**: Effective identity of a schema resource (`$id` when present,
    /// otherwise `schema_document_uri`).
    pub schema_resource_uri: SchemaUri,
    pub id: Option<SchemaUri>,
    pub dialect: Option<JsonSchemaDialect>,
    pub validation_vocabulary_disabled: bool,
    /// JSON Pointer to this resource within the physical document (e.g. `#/$defs/foo`).
    pub location: String,
    parent_resource_uri: Option<SchemaUri>,
    has_schema_keyword: bool,
}

#[derive(Debug)]
pub(crate) struct SchemaDocumentResources {
    /// **schema_document_uri**: Physical URI of the loaded JSON Schema document (file/http/etc).
    /// Not `$id`.
    schema_document_uri: SchemaUri,
    root_schema_resource_uri: SchemaUri,
    root: tombi_json::ValueNode,
    /// The line index of the physical document, to convert the spans of its schemas.
    line_index: Arc<tombi_text::OwnedLineIndex>,
    resources: tombi_hashmap::HashMap<SchemaUri, SchemaResource>,
}

impl SchemaDocumentResources {
    pub(crate) async fn collect(
        document: tombi_json::Document,
        schema_document_uri: &SchemaUri,
        schema_store: &SchemaStore,
    ) -> Result<Arc<Self>, crate::Error> {
        let tombi_json::Document {
            value: root,
            line_index,
        } = document;
        let (root_dialect, root_validation_vocabulary_disabled) =
            root_schema_context(&root, schema_document_uri, schema_store).await;
        let mut resources = tombi_hashmap::HashMap::default();
        let root_schema_resource_uri = collect_schema_resources_from_value(
            &root,
            schema_document_uri,
            schema_document_uri,
            root_dialect,
            root_validation_vocabulary_disabled,
            true,
            "#",
            &mut resources,
        )?
        .expect("the root schema always defines a schema resource");
        inherit_resource_contexts(&mut resources, &root, schema_document_uri, schema_store).await;

        if schema_document_uri != &root_schema_resource_uri
            && let Some(existing) = resources.get(schema_document_uri)
        {
            return Err(crate::Error::DuplicateSchemaResourceInDocument(Box::new(
                crate::error::DuplicateSchemaResourceInDocument {
                    schema_uri: schema_document_uri.clone(),
                    schema_document_uri: schema_document_uri.clone(),
                    first_location: existing.location.clone(),
                    second_location: "#".to_string(),
                },
            )));
        }

        Ok(Arc::new(Self {
            schema_document_uri: schema_document_uri.clone(),
            root_schema_resource_uri,
            root,
            line_index,
            resources,
        }))
    }

    pub(crate) fn schema_document_uri(&self) -> &SchemaUri {
        &self.schema_document_uri
    }

    pub(crate) fn root_schema_resource_uri(&self) -> &SchemaUri {
        &self.root_schema_resource_uri
    }

    pub(crate) fn line_index(&self) -> &Arc<tombi_text::OwnedLineIndex> {
        &self.line_index
    }

    pub(crate) fn resource(&self, schema_resource_uri: &SchemaUri) -> Option<&SchemaResource> {
        self.resources.get(schema_resource_uri)
    }

    pub(crate) fn resource_value(
        &self,
        resource: &SchemaResource,
    ) -> Option<&tombi_json::ValueNode> {
        resource_value_at_location(&self.root, &resource.location)
    }

    pub(crate) fn resource_uris(&self) -> impl Iterator<Item = &SchemaUri> + '_ {
        self.resources.keys()
    }
}

async fn root_schema_context(
    root: &tombi_json::ValueNode,
    schema_document_uri: &SchemaUri,
    schema_store: &SchemaStore,
) -> (Option<JsonSchemaDialect>, bool) {
    let Some(object) = root.as_object() else {
        return (None, false);
    };
    let Some(schema_uri) = object
        .get("$schema")
        .and_then(tombi_json::ValueNode::as_str)
    else {
        return (None, false);
    };
    if let Ok(dialect) = JsonSchemaDialect::try_from(schema_uri) {
        return (Some(dialect), false);
    }

    let schema_base_uri = object
        .get("$id")
        .and_then(tombi_json::ValueNode::as_str)
        .and_then(|id| resolve_schema_resource_uri(schema_document_uri, id))
        .unwrap_or_else(|| schema_document_uri.clone());
    let Some(metaschema_uri) = resolve_schema_resource_uri(&schema_base_uri, schema_uri) else {
        return (None, false);
    };
    let Ok(Some(tombi_json::Document {
        value: tombi_json::ValueNode::Object(metaschema),
        ..
    })) = schema_store.fetch_schema_document(&metaschema_uri).await
    else {
        return (None, false);
    };
    let dialect = metaschema
        .get("$schema")
        .and_then(tombi_json::ValueNode::as_str)
        .and_then(|schema| JsonSchemaDialect::try_from(schema).ok());
    let validation_vocabulary_disabled =
        validation_vocabulary_is_disabled_in_object(&metaschema, dialect);
    (dialect, validation_vocabulary_disabled)
}

async fn inherit_resource_contexts(
    resources: &mut tombi_hashmap::HashMap<SchemaUri, SchemaResource>,
    root: &tombi_json::ValueNode,
    schema_document_uri: &SchemaUri,
    schema_store: &SchemaStore,
) {
    if resources.len() <= 1
        || !resources
            .values()
            .any(|resource| resource.location != "#" && resource.has_schema_keyword)
    {
        return;
    }

    let mut resource_uris = resources.keys().cloned().collect::<Vec<_>>();
    resource_uris.sort_by_key(|uri| {
        resources
            .get(uri)
            .map(|resource| resource.location.matches('/').count())
            .unwrap_or_default()
    });

    for resource_uri in resource_uris {
        let Some(resource) = resources.get(&resource_uri).cloned() else {
            continue;
        };
        if resource.location == "#" {
            continue;
        }
        let Some((inherited_dialect, inherited_disabled)) = resource
            .parent_resource_uri
            .as_ref()
            .and_then(|uri| resources.get(uri))
            .map(|parent| (parent.dialect, parent.validation_vocabulary_disabled))
        else {
            continue;
        };
        let (dialect, disabled) = if resource.has_schema_keyword {
            if let Some(value) = resource_value_at_location(root, &resource.location) {
                let base_uri = resource.id.as_ref().unwrap_or(schema_document_uri);
                root_schema_context(value, base_uri, schema_store).await
            } else {
                continue;
            }
        } else {
            (inherited_dialect, inherited_disabled)
        };
        if let Some(resource) = resources.get_mut(&resource_uri) {
            resource.dialect = dialect;
            resource.validation_vocabulary_disabled = disabled;
        }
    }
}

fn validation_vocabulary_is_disabled_in_object(
    object: &tombi_json::ObjectNode,
    dialect: Option<JsonSchemaDialect>,
) -> bool {
    let vocabulary_uris = match dialect {
        Some(JsonSchemaDialect::Draft2019_09) => {
            &["https://json-schema.org/draft/2019-09/vocab/validation"][..]
        }
        Some(JsonSchemaDialect::Draft2020_12) => {
            &["https://json-schema.org/draft/2020-12/vocab/validation"][..]
        }
        Some(JsonSchemaDialect::Draft07) => return false,
        None => &[
            "https://json-schema.org/draft/2019-09/vocab/validation",
            "https://json-schema.org/draft/2020-12/vocab/validation",
        ][..],
    };

    object
        .get("$vocabulary")
        .and_then(|v| v.as_object())
        .is_some_and(|vocab| {
            vocabulary_uris.iter().all(|uri| {
                !vocab.get(uri).is_some_and(
                    |value| matches!(value, tombi_json::ValueNode::Bool(value) if value.value),
                )
            })
        })
}

fn collect_schema_resources_from_value(
    value: &tombi_json::ValueNode,
    schema_document_uri: &SchemaUri,
    enclosing_schema_base_uri: &SchemaUri,
    inherited_dialect: Option<JsonSchemaDialect>,
    inherited_validation_vocabulary_disabled: bool,
    is_document_root: bool,
    location: &str,
    resources: &mut tombi_hashmap::HashMap<SchemaUri, SchemaResource>,
) -> Result<Option<SchemaUri>, crate::Error> {
    let tombi_json::ValueNode::Object(object) = value else {
        if is_document_root {
            let schema_resource_uri = schema_document_uri.clone();
            resources.insert(
                schema_resource_uri.clone(),
                SchemaResource {
                    schema_resource_uri: schema_resource_uri.clone(),
                    id: None,
                    dialect: inherited_dialect,
                    validation_vocabulary_disabled: inherited_validation_vocabulary_disabled,
                    location: location.to_string(),
                    parent_resource_uri: None,
                    has_schema_keyword: false,
                },
            );
            return Ok(Some(schema_resource_uri));
        }
        return Ok(None);
    };

    let declared_schema_resource_uri = object
        .get("$id")
        .and_then(tombi_json::ValueNode::as_str)
        .and_then(|id| resolve_schema_resource_uri(enclosing_schema_base_uri, id));
    let dialect = (is_document_root || declared_schema_resource_uri.is_some())
        .then(|| {
            object
                .get("$schema")
                .and_then(tombi_json::ValueNode::as_str)
                .and_then(|uri| JsonSchemaDialect::try_from(uri).ok())
        })
        .flatten()
        .or(inherited_dialect);
    let validation_vocabulary_disabled = inherited_validation_vocabulary_disabled;
    let schema_resource_uri = declared_schema_resource_uri
        .clone()
        .or_else(|| is_document_root.then(|| schema_document_uri.clone()));
    let schema_base_uri = declared_schema_resource_uri
        .as_ref()
        .unwrap_or(enclosing_schema_base_uri);

    if let Some(schema_resource_uri) = &schema_resource_uri {
        if let Some(existing) = resources.get(schema_resource_uri) {
            return Err(crate::Error::DuplicateSchemaResourceInDocument(Box::new(
                crate::error::DuplicateSchemaResourceInDocument {
                    schema_uri: schema_resource_uri.clone(),
                    schema_document_uri: schema_document_uri.clone(),
                    first_location: existing.location.clone(),
                    second_location: location.to_string(),
                },
            )));
        }
        resources.insert(
            schema_resource_uri.clone(),
            SchemaResource {
                schema_resource_uri: schema_resource_uri.clone(),
                id: declared_schema_resource_uri.clone(),
                dialect,
                validation_vocabulary_disabled,
                location: location.to_string(),
                parent_resource_uri: (!is_document_root).then(|| enclosing_schema_base_uri.clone()),
                has_schema_keyword: object.get("$schema").is_some(),
            },
        );
    }

    for (key, child) in object.properties.iter() {
        match key.value.as_str() {
            "$defs" | "definitions" | "properties" | "patternProperties" | "dependentSchemas" => {
                if let Some(children) = child.as_object() {
                    for (child_key, child) in children.properties.iter() {
                        collect_schema_resources_from_value(
                            child,
                            schema_document_uri,
                            schema_base_uri,
                            dialect,
                            validation_vocabulary_disabled,
                            false,
                            &json_pointer_join(
                                location,
                                key.value.as_str(),
                                Some(&child_key.value),
                            ),
                            resources,
                        )?;
                    }
                }
            }
            "dependencies" => {
                if let Some(children) = child.as_object() {
                    for (child_key, child) in children.properties.iter().filter(|(_, value)| {
                        matches!(
                            value,
                            tombi_json::ValueNode::Object(_) | tombi_json::ValueNode::Bool(_)
                        )
                    }) {
                        collect_schema_resources_from_value(
                            child,
                            schema_document_uri,
                            schema_base_uri,
                            dialect,
                            validation_vocabulary_disabled,
                            false,
                            &json_pointer_join(
                                location,
                                key.value.as_str(),
                                Some(&child_key.value),
                            ),
                            resources,
                        )?;
                    }
                }
            }
            "allOf" | "anyOf" | "oneOf" | "prefixItems" => {
                if let Some(children) = child.as_array() {
                    for (index, child) in children.items.iter().enumerate() {
                        collect_schema_resources_from_value(
                            child,
                            schema_document_uri,
                            schema_base_uri,
                            dialect,
                            validation_vocabulary_disabled,
                            false,
                            &json_pointer_join(
                                location,
                                key.value.as_str(),
                                Some(&index.to_string()),
                            ),
                            resources,
                        )?;
                    }
                }
            }
            "items" => {
                if let Some(children) = child.as_array() {
                    for (index, child) in children.items.iter().enumerate() {
                        collect_schema_resources_from_value(
                            child,
                            schema_document_uri,
                            schema_base_uri,
                            dialect,
                            validation_vocabulary_disabled,
                            false,
                            &json_pointer_join(
                                location,
                                key.value.as_str(),
                                Some(&index.to_string()),
                            ),
                            resources,
                        )?;
                    }
                } else {
                    collect_schema_resources_from_value(
                        child,
                        schema_document_uri,
                        schema_base_uri,
                        dialect,
                        validation_vocabulary_disabled,
                        false,
                        &json_pointer_join(location, key.value.as_str(), None),
                        resources,
                    )?;
                }
            }
            "additionalItems"
            | "additionalProperties"
            | "unevaluatedItems"
            | "unevaluatedProperties"
            | "contains"
            | "propertyNames"
            | "not"
            | "if"
            | "then"
            | "else"
            | "contentSchema" => {
                collect_schema_resources_from_value(
                    child,
                    schema_document_uri,
                    schema_base_uri,
                    dialect,
                    validation_vocabulary_disabled,
                    false,
                    &json_pointer_join(location, key.value.as_str(), None),
                    resources,
                )?;
            }
            _ => {}
        }
    }

    Ok(schema_resource_uri)
}

fn json_pointer_join(parent: &str, key: &str, child: Option<&str>) -> String {
    let mut pointer = if parent == "#" {
        format!(
            "#/{}",
            crate::keyword_support::escape_json_pointer_token(key)
        )
    } else {
        format!(
            "{parent}/{}",
            crate::keyword_support::escape_json_pointer_token(key)
        )
    };
    if let Some(child) = child {
        pointer.push('/');
        pointer.push_str(&crate::keyword_support::escape_json_pointer_token(child));
    }
    pointer
}

fn resource_value_at_location<'a>(
    root: &'a tombi_json::ValueNode,
    location: &str,
) -> Option<&'a tombi_json::ValueNode> {
    if location == "#" {
        return Some(root);
    }

    let mut value = root;
    for token in location.strip_prefix("#/")?.split('/') {
        let token = if token.contains('~') {
            std::borrow::Cow::Owned(token.replace("~1", "/").replace("~0", "~"))
        } else {
            std::borrow::Cow::Borrowed(token)
        };
        value = match value {
            tombi_json::ValueNode::Object(object) => object.get(token.as_ref())?,
            tombi_json::ValueNode::Array(array) => array.get(token.parse().ok()?)?,
            _ => return None,
        };
    }
    Some(value)
}

pub(crate) fn resolve_schema_resource_uri(
    schema_base_uri: &SchemaUri,
    id: &str,
) -> Option<SchemaUri> {
    let mut schema_resource_uri = if let Ok(uri) = schema_base_uri.join(id) {
        SchemaUri::from(uri)
    } else {
        SchemaUri::from_str(id).ok()?
    };
    if schema_resource_uri
        .fragment()
        .is_some_and(|fragment| !fragment.is_empty())
    {
        return None;
    }
    schema_resource_uri.set_fragment(None);
    Some(schema_resource_uri)
}
