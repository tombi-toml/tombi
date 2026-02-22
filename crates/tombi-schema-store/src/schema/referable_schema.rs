use std::{borrow::Cow, str::FromStr, sync::Arc};

use itertools::Itertools;
use tombi_x_keyword::StringFormat;

use crate::x_taplo::XTaplo;

use super::{SchemaDefinitions, SchemaUri, ValueSchema};

#[derive(Debug, Clone, PartialEq)]
pub enum Referable<T> {
    Resolved {
        schema_uri: Option<SchemaUri>,
        value: Arc<T>,
    },
    Ref {
        reference: String,
        title: Option<String>,
        description: Option<String>,
        deprecated: Option<bool>,
    },
}

#[derive(Clone)]
pub struct CurrentSchema<'a> {
    pub value_schema: Arc<ValueSchema>,
    pub schema_uri: Cow<'a, SchemaUri>,
    pub definitions: Cow<'a, SchemaDefinitions>,
}

impl<'a> CurrentSchema<'a> {
    pub fn into_owned(self) -> CurrentSchema<'static> {
        CurrentSchema {
            value_schema: self.value_schema,
            schema_uri: Cow::Owned(self.schema_uri.into_owned()),
            definitions: Cow::Owned(self.definitions.into_owned()),
        }
    }
}

impl std::fmt::Debug for CurrentSchema<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CurrentSchema")
            .field("value_schema", &self.value_schema)
            .field("schema_uri", &self.schema_uri.to_string())
            .finish()
    }
}

impl<T> Referable<T> {
    pub fn resolved(&self) -> Option<&T> {
        match self {
            Self::Resolved { value, .. } => Some(value.as_ref()),
            Self::Ref { .. } => None,
        }
    }
}

impl Referable<ValueSchema> {
    pub fn new(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
    ) -> Option<Self> {
        if let Some(x_taplo) = object.get("x-taplo")
            && let Ok(x_taplo) = tombi_json::from_value_node::<XTaplo>(x_taplo.to_owned())
            && x_taplo.hidden == Some(true)
        {
            return None;
        }
        if let Some(tombi_json::ValueNode::String(ref_string)) = object.get("$ref") {
            return Some(Referable::Ref {
                reference: ref_string.value.clone(),
                title: object
                    .get("title")
                    .and_then(|title| title.as_str().map(|s| s.to_string())),
                description: object
                    .get("description")
                    .and_then(|description| description.as_str().map(|s| s.to_string())),
                deprecated: object
                    .get("deprecated")
                    .and_then(|deprecated| deprecated.as_bool()),
            });
        }

        ValueSchema::new(object, string_formats).map(|value_schema| Referable::Resolved {
            schema_uri: None,
            value: Arc::new(value_schema),
        })
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self, Referable::Resolved { .. })
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Referable::Ref { .. })
    }

    pub fn deprecated<'a: 'b, 'b>(&'a self) -> tombi_future::BoxFuture<'b, Option<bool>> {
        Box::pin(async move {
            match self {
                Referable::Resolved { value, .. } => value.deprecated().await,
                Referable::Ref { .. } => None,
            }
        })
    }

    pub async fn value_type(&self) -> crate::ValueType {
        match self {
            Referable::Resolved { value, .. } => value.value_type().await,
            Referable::Ref { reference, .. } => {
                log::warn!(
                    "unresolved $ref while determining value type: reference={}",
                    reference
                );
                // Unknown under the current API surface (no schema context here).
                crate::ValueType::AnyOf(Vec::new())
            }
        }
    }

    pub fn resolve<'a: 'b, 'b>(
        &'a mut self,
        schema_uri: Cow<'a, SchemaUri>,
        definitions: Cow<'a, SchemaDefinitions>,
        schema_store: &'a crate::SchemaStore,
    ) -> tombi_future::BoxFuture<'b, Result<Option<CurrentSchema<'a>>, crate::Error>> {
        Box::pin(async move {
            match self {
                Referable::Ref {
                    reference,
                    title,
                    description,
                    deprecated,
                } => {
                    let definition_schema = {
                        let definitions_guard = definitions.read().await;
                        definitions_guard.get(reference).cloned()
                    };
                    if let Some(definition_schema) = definition_schema {
                        let mut referable_schema = definition_schema;
                        if let Referable::Resolved {
                            value: value_schema,
                            ..
                        } = &mut referable_schema
                        {
                            let value_schema = Arc::make_mut(value_schema);
                            if title.is_some() || description.is_some() {
                                value_schema.set_title(title.to_owned());
                                value_schema.set_description(description.to_owned());
                            }
                            if let Some(deprecated) = deprecated {
                                value_schema.set_deprecated(*deprecated);
                            }
                        }

                        *self = referable_schema;
                    } else if is_json_pointer(reference) {
                        let pointer = reference;

                        // Exceptional handling for schemas that do not use `#/definitions/*`.
                        // Therefore, schema_value is not cached in memory, but read from file cache.
                        // Execution speed decreases, but memory usage can be reduced.
                        if let Some(schema_value) =
                            schema_store.fetch_schema_value(&schema_uri).await?
                        {
                            if let Some(mut resolved_schema) =
                                resolve_json_pointer(&schema_value, pointer, None)?
                            {
                                if title.is_some() || description.is_some() {
                                    resolved_schema.set_title(title.to_owned());
                                    resolved_schema.set_description(description.to_owned());
                                }
                                if let Some(deprecated) = deprecated {
                                    resolved_schema.set_deprecated(*deprecated);
                                }

                                return Ok(Some(CurrentSchema {
                                    value_schema: Arc::new(resolved_schema),
                                    schema_uri: Cow::Owned(schema_uri.as_ref().clone()),
                                    definitions: Cow::Owned(definitions.clone().into_owned()),
                                }));
                            } else {
                                return Err(crate::Error::InvalidJsonPointer {
                                    pointer: pointer.to_owned(),
                                    schema_uri: schema_uri.as_ref().clone(),
                                });
                            }
                        } else {
                            // Offline Mode
                            return Ok(None);
                        }
                    } else if let Ok(schema_uri) = SchemaUri::from_str(reference) {
                        if let Some(document_schema) =
                            schema_store.try_get_document_schema(&schema_uri).await?
                        {
                            if let Some(value_schema) = document_schema.value_schema.as_ref() {
                                let mut resolved_value = value_schema.clone();
                                if title.is_some() || description.is_some() {
                                    let value_schema = Arc::make_mut(&mut resolved_value);
                                    value_schema.set_title(title.to_owned());
                                    value_schema.set_description(description.to_owned());
                                }
                                if let Some(deprecated) = deprecated {
                                    let value_schema = Arc::make_mut(&mut resolved_value);
                                    value_schema.set_deprecated(*deprecated);
                                }

                                *self = Referable::Resolved {
                                    schema_uri: Some(document_schema.schema_uri.clone()),
                                    value: resolved_value,
                                };

                                return self
                                    .resolve(
                                        Cow::Owned(document_schema.schema_uri.clone()),
                                        Cow::Owned(document_schema.definitions.clone()),
                                        schema_store,
                                    )
                                    .await;
                            } else {
                                return Err(crate::Error::InvalidJsonSchemaReference {
                                    reference: reference.to_owned(),
                                    schema_uri: schema_uri.clone(),
                                });
                            }
                        } else {
                            return Ok(None);
                        }
                    } else {
                        return Err(crate::Error::UnsupportedReference {
                            reference: reference.to_owned(),
                            schema_uri: schema_uri.as_ref().to_owned(),
                        });
                    }

                    self.resolve(schema_uri, definitions, schema_store).await
                }
                Referable::Resolved {
                    schema_uri: reference_url,
                    value: value_schema,
                    ..
                } => {
                    let (schema_uri, definitions) = {
                        match reference_url {
                            Some(reference_url) => {
                                if let Some(document_schema) =
                                    schema_store.try_get_document_schema(reference_url).await?
                                {
                                    (
                                        Cow::Owned(document_schema.schema_uri.clone()),
                                        Cow::Owned(document_schema.definitions.clone()),
                                    )
                                } else {
                                    (schema_uri, definitions)
                                }
                            }
                            None => (schema_uri, definitions),
                        }
                    };

                    Ok(Some(CurrentSchema {
                        value_schema: value_schema.clone(),
                        schema_uri,
                        definitions,
                    }))
                }
            }
        })
    }

    /// Constructs a `CurrentSchema<'static>` from a `Resolved` variant without mutation.
    /// Returns `Ok(None)` for `Ref` variants (they need `resolve()` first).
    ///
    /// This is designed for use under a read lock, where we've already confirmed
    /// all schemas are Resolved.
    pub async fn to_current_schema(
        &self,
        schema_uri: Cow<'_, SchemaUri>,
        definitions: Cow<'_, SchemaDefinitions>,
        schema_store: &crate::SchemaStore,
    ) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
        match self {
            Referable::Ref { .. } => Ok(None),
            Referable::Resolved {
                schema_uri: reference_url,
                value: value_schema,
            } => {
                let (schema_uri, definitions) = match reference_url {
                    Some(reference_url) => {
                        if let Some(document_schema) =
                            schema_store.try_get_document_schema(reference_url).await?
                        {
                            (
                                Cow::Owned(document_schema.schema_uri.clone()),
                                Cow::Owned(document_schema.definitions.clone()),
                            )
                        } else {
                            (schema_uri, definitions)
                        }
                    }
                    None => (schema_uri, definitions),
                };

                Ok(Some(CurrentSchema {
                    value_schema: value_schema.clone(),
                    schema_uri: Cow::Owned(schema_uri.into_owned()),
                    definitions: Cow::Owned(definitions.into_owned()),
                }))
            }
        }
    }
}

/// Two-path schema collection: tries a read lock first for already-resolved schemas,
/// resolves refs on cloned entries, and writes back only newly-resolved entries.
///
/// Returns `None` when schema traversal is re-entrant (cycle guard) or when
/// an initial read lock cannot be acquired due to concurrent mutation.
pub async fn resolve_and_collect_schemas(
    schemas: &super::ReferableValueSchemas,
    schema_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    schema_store: &crate::SchemaStore,
    schema_visits: &crate::SchemaVisits,
    accessors: &[crate::Accessor],
) -> Option<Vec<CurrentSchema<'static>>> {
    let Some(_cycle_guard) = schema_visits.get_cycle_guard(schemas) else {
        log::debug!(
            "detected composite schema cycle while collecting schemas: schema_uri={schema_uri} accessors={accessors} reason=reentrant_schema_traversal",
            schema_uri = schema_uri.as_ref().to_string(),
            accessors = crate::Accessors::from(accessors.to_vec())
        );
        return None;
    };

    let mut schema_entries = Vec::new();
    let resolved_schemas = {
        let Ok(schema_guard) = schemas.try_read() else {
            // try_read() failed -- a write lock is held.
            log::debug!(
                "failed to acquire read lock for composite schema collection: schema_uri={schema_uri} accessors={accessors} reason=write_lock_held",
                schema_uri = schema_uri.as_ref().to_string(),
                accessors = crate::Accessors::from(accessors.to_vec())
            );
            return None;
        };

        if schema_guard.iter().all(Referable::is_resolved) {
            Some(
                schema_guard
                    .iter()
                    .filter_map(|referable_schema| match referable_schema {
                        Referable::Resolved {
                            schema_uri: resolved_schema_uri,
                            value,
                        } => Some((resolved_schema_uri.clone(), value.clone())),
                        Referable::Ref { .. } => None,
                    })
                    .collect_vec(),
            )
        } else {
            schema_entries = schema_guard.clone();
            None
        }
    };

    // Fast path: all schemas are already resolved.
    // Build output from read result and avoid cloning the whole referable vector.
    if let Some(resolved_schemas) = resolved_schemas {
        let mut collected = Vec::with_capacity(resolved_schemas.len());
        let default_schema_uri = schema_uri.as_ref().clone();
        let default_definitions = definitions.clone().into_owned();

        for (resolved_schema_uri, value_schema) in resolved_schemas {
            let (current_schema_uri, current_definitions) =
                if let Some(resolved_schema_uri) = resolved_schema_uri {
                    match schema_store
                        .try_get_document_schema(&resolved_schema_uri)
                        .await
                    {
                        Ok(Some(document_schema)) => (
                            document_schema.schema_uri.clone(),
                            document_schema.definitions.clone(),
                        ),
                        Ok(None) => (default_schema_uri.clone(), default_definitions.clone()),
                        Err(err) => {
                            log::warn!("{err}");
                            continue;
                        }
                    }
                } else {
                    (default_schema_uri.clone(), default_definitions.clone())
                };

            collected.push(CurrentSchema {
                value_schema,
                schema_uri: Cow::Owned(current_schema_uri),
                definitions: Cow::Owned(current_definitions),
            });
        }

        return Some(collected);
    }

    // Slow path: unresolved refs exist. Resolve on cloned entries and cache back.
    let mut collected = Vec::with_capacity(schema_entries.len());
    let mut resolved_indices = Vec::new();
    for (index, referable_schema) in schema_entries.iter_mut().enumerate() {
        let was_ref = referable_schema.is_ref();
        match referable_schema
            .resolve(schema_uri.clone(), definitions.clone(), schema_store)
            .await
        {
            Ok(Some(current_schema)) => collected.push(current_schema.into_owned()),
            Ok(None) => {}
            Err(err) => {
                log::warn!("{err}");
            }
        }

        if was_ref && referable_schema.is_resolved() {
            resolved_indices.push(index);
        }
    }

    // Write back only entries that transitioned from Ref -> Resolved.
    if !resolved_indices.is_empty() {
        let Ok(mut schema_guard) = schemas.try_write() else {
            log::debug!(
                "failed to acquire write lock for composite schema resolution: schema_uri={schema_uri} accessors={accessors} reason=lock_contention",
                schema_uri = schema_uri.as_ref().to_string(),
                accessors = crate::Accessors::from(accessors.to_vec())
            );
            return Some(collected);
        };

        for index in resolved_indices {
            if let (Some(cached_schema), Some(resolved_schema)) =
                (schema_guard.get_mut(index), schema_entries.get(index))
                && cached_schema.is_ref()
                && resolved_schema.is_resolved()
            {
                *cached_schema = resolved_schema.clone();
            }
        }
    }

    Some(collected)
}

/// Resolve a schema item without holding its write lock across await points.
///
/// 1. Clone under read lock.
/// 2. If already resolved, build `CurrentSchema` directly.
/// 3. If unresolved, resolve on the cloned item.
/// 4. Write back only the resolved cache state.
pub async fn resolve_schema_item(
    item: &super::SchemaItem,
    schema_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    schema_store: &crate::SchemaStore,
) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
    let mut item_schema = {
        let item_schema = item.read().await;
        if item_schema.is_resolved() {
            return item_schema
                .to_current_schema(schema_uri, definitions, schema_store)
                .await;
        }
        item_schema.clone()
    };

    let resolved = item_schema
        .resolve(schema_uri.clone(), definitions.clone(), schema_store)
        .await?
        .map(CurrentSchema::into_owned);

    if item_schema.is_resolved() {
        let mut new_item_schema = item.write().await;
        if new_item_schema.is_ref() {
            *new_item_schema = item_schema;
        }
    }

    Ok(resolved)
}

pub fn is_online_url(reference: &str) -> bool {
    reference.starts_with("https://") || reference.starts_with("http://")
}

pub fn is_json_pointer(reference: &str) -> bool {
    reference.starts_with('#')
}

/// Resolve a JSON pointer to a ValueSchema.
///
/// This function resolves a JSON pointer to a ValueSchema.
/// It is used to resolve pointers like `#/properties/foo` within the same schema.
/// More correctly, it should use `#/definitions/foo` to use definitions,
/// but this function is provided for exceptional cases of some JSON Schema implementations.
///
pub fn resolve_json_pointer(
    schema_node: &tombi_json::ValueNode,
    pointer: &str,
    string_formats: Option<&[StringFormat]>,
) -> Result<Option<ValueSchema>, crate::Error> {
    if !pointer.starts_with('#') {
        return Ok(None);
    }

    let path = &pointer[1..]; // Remove the leading '#'
    if path.is_empty() {
        return Ok(schema_node
            .as_object()
            .and_then(|obj| ValueSchema::new(obj, string_formats)));
    }

    // RFC 6901: Percent-decode the path before splitting on '/'
    let decoded_path = percent_decode(path);
    let segments: Vec<&str> = decoded_path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current = schema_node;

    for segment in segments {
        let decoded_segment = segment.replace("~1", "/").replace("~0", "~");

        match current {
            tombi_json::ValueNode::Object(obj) => {
                if let Some(value) = obj.get(&decoded_segment) {
                    current = value;
                } else {
                    return Ok(None);
                }
            }
            tombi_json::ValueNode::Array(arr) => {
                if let Ok(index) = decoded_segment.parse::<usize>() {
                    if let Some(value) = arr.get(index) {
                        current = value;
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            _ => {
                return Ok(None);
            }
        }
    }

    // Convert the final ValueNode to ValueSchema
    match current {
        tombi_json::ValueNode::Object(obj) => Ok(ValueSchema::new(obj, string_formats)),
        _ => Ok(None),
    }
}

/// Percent-decode a string according to RFC 3986
fn percent_decode(input: &str) -> String {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            // Look ahead for two hex digits
            let mut hex_chars = String::new();
            for _ in 0..2 {
                if let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_hexdigit() {
                        hex_chars.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if hex_chars.len() == 2
                && let Ok(byte) = u8::from_str_radix(&hex_chars, 16)
            {
                result.push(byte);
                continue;
            }

            // If percent decoding failed, keep the original '%' and hex chars
            result.extend_from_slice(b"%");
            result.extend_from_slice(hex_chars.as_bytes());
        } else {
            result.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
        }
    }

    // Convert bytes back to string, handling invalid UTF-8 gracefully
    String::from_utf8_lossy(&result).into_owned()
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{Referable, ValueSchema, schema::referable_schema::resolve_json_pointer};

    #[test]
    fn test_json_pointer_percent_decode() {
        use tombi_json::ValueNode;

        // Test case 1: Basic percent decoding
        let json = r#"{
            "foo": {
                "bar%2Fbaz": "value1",
                "qux": "value2"
            }
        }"#;
        let value_node = ValueNode::from_str(json).unwrap();

        // Test with percent-encoded slash
        let result = resolve_json_pointer(&value_node, "#/foo/bar%2Fbaz", None);
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            // The schema should be resolved correctly
            assert!(matches!(schema, ValueSchema::String(_)));
        }

        // Test case 2: Multiple percent-encoded characters
        let json = r#"{
            "test": {
                "path%2Fwith%20spaces": "value"
            }
        }"#;
        let value_node = ValueNode::from_str(json).unwrap();

        let result = resolve_json_pointer(&value_node, "#/test/path%2Fwith%20spaces", None);
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            assert!(matches!(schema, ValueSchema::String(_)));
        }

        // Test case 3: Invalid percent encoding should be preserved
        let json = r#"{
            "foo": {
                "bar%2": "value1",
                "baz%2G": "value2"
            }
        }"#;
        let value_node = ValueNode::from_str(json).unwrap();

        // These should return None because the keys don't exist after failed decoding
        let result = resolve_json_pointer(&value_node, "#/foo/bar%2", None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let result = resolve_json_pointer(&value_node, "#/foo/baz%2G", None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Test case 4: Mixed with JSON pointer escape sequences
        let json = r#"{
            "foo": {
                "bar~1baz": "value1",
                "qux~0tilde": "value2"
            }
        }"#;
        let value_node = ValueNode::from_str(json).unwrap();

        // Test JSON pointer escape sequences (should work as before)
        let result = resolve_json_pointer(&value_node, "#/foo/bar~1baz", None);
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            assert!(matches!(schema, ValueSchema::String(_)));
        }

        let result = resolve_json_pointer(&value_node, "#/foo/qux~0tilde", None);
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            assert!(matches!(schema, ValueSchema::String(_)));
        }
    }

    #[tokio::test]
    async fn test_value_type_ref_does_not_panic() {
        let referable = Referable::Ref {
            reference: "#/definitions/foo".to_string(),
            title: None,
            description: None,
            deprecated: None,
        };

        let value_type = referable.value_type().await;
        assert!(matches!(value_type, crate::ValueType::AnyOf(types) if types.is_empty()));
    }
}
