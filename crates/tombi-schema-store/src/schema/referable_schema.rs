use std::borrow::Cow;

use crate::x_taplo::XTaplo;

use super::{AllOfSchema, AnyOfSchema, OneOfSchema, SchemaDefinitions, SchemaUrl, ValueSchema};

#[derive(Debug, Clone, PartialEq)]
pub enum Referable<T> {
    Resolved {
        schema_url: Option<SchemaUrl>,
        value: T,
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
    pub value_schema: Cow<'a, ValueSchema>,
    pub schema_url: Cow<'a, SchemaUrl>,
    pub definitions: Cow<'a, SchemaDefinitions>,
}

impl std::fmt::Debug for CurrentSchema<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CurrentSchema")
            .field("value_schema", &self.value_schema)
            .field("schema_url", &self.schema_url.to_string())
            .finish()
    }
}

impl<T> Referable<T> {
    pub fn resolved(&self) -> Option<&T> {
        match self {
            Self::Resolved { value, .. } => Some(value),
            Self::Ref { .. } => None,
        }
    }
}

impl Referable<ValueSchema> {
    pub fn new(object: &tombi_json::ObjectNode) -> Option<Self> {
        if let Some(x_taplo) = object.get("x-taplo") {
            if let Ok(x_taplo) = tombi_json::from_value_node::<XTaplo>(x_taplo.to_owned()) {
                if x_taplo.hidden == Some(true) {
                    return None;
                }
            }
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

        ValueSchema::new(object).map(|value_schema| Referable::Resolved {
            schema_url: None,
            value: value_schema,
        })
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
            Referable::Ref { .. } => unreachable!("unreachable ref value_tyle."),
        }
    }

    pub fn resolve<'a: 'b, 'b>(
        &'a mut self,
        schema_url: Cow<'a, SchemaUrl>,
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
                    if let Some(definition_schema) = definitions.read().await.get(reference) {
                        let mut referable_schema = definition_schema.to_owned();
                        if let Referable::Resolved {
                            value: ref mut value_schema,
                            ..
                        } = &mut referable_schema
                        {
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
                            schema_store.fetch_schema_value(&schema_url).await?
                        {
                            if let Some(mut resolved_schema) =
                                resolve_json_pointer(&schema_value, pointer)?
                            {
                                if title.is_some() || description.is_some() {
                                    resolved_schema.set_title(title.to_owned());
                                    resolved_schema.set_description(description.to_owned());
                                }
                                if let Some(deprecated) = deprecated {
                                    resolved_schema.set_deprecated(*deprecated);
                                }

                                *self = Referable::Resolved {
                                    schema_url: Some(schema_url.as_ref().clone()),
                                    value: resolved_schema,
                                };
                            } else {
                                return Err(crate::Error::InvalidJsonPointer {
                                    pointer: pointer.to_owned(),
                                    schema_url: schema_url.as_ref().clone(),
                                });
                            }
                        } else {
                            // Offline Mode
                            return Ok(None);
                        }
                    } else if is_online_url(reference) {
                        let schema_url = SchemaUrl::parse(reference)?;

                        if let Some(mut document_schema) =
                            schema_store.try_get_document_schema(&schema_url).await?
                        {
                            if let Some(value_schema) = &mut document_schema.value_schema {
                                if title.is_some() || description.is_some() {
                                    value_schema.set_title(title.to_owned());
                                    value_schema.set_description(description.to_owned());
                                }
                                if let Some(deprecated) = deprecated {
                                    value_schema.set_deprecated(*deprecated);
                                }

                                *self = Referable::Resolved {
                                    schema_url: Some(document_schema.schema_url.clone()),
                                    value: value_schema.clone(),
                                };

                                return self
                                    .resolve(
                                        Cow::Owned(document_schema.schema_url),
                                        Cow::Owned(document_schema.definitions),
                                        schema_store,
                                    )
                                    .await;
                            } else {
                                return Err(crate::Error::InvalidJsonSchemaReference {
                                    reference: reference.to_owned(),
                                    schema_url: schema_url.clone(),
                                });
                            }
                        } else {
                            return Ok(None);
                        }
                    } else {
                        return Err(crate::Error::UnsupportedReference {
                            reference: reference.to_owned(),
                            schema_url: schema_url.as_ref().clone(),
                        });
                    }

                    self.resolve(schema_url, definitions, schema_store).await
                }
                Referable::Resolved {
                    schema_url: reference_url,
                    value: value_schema,
                    ..
                } => {
                    let (schema_url, definitions) = {
                        match reference_url {
                            Some(reference_url) => {
                                if let Some(document_schema) =
                                    schema_store.try_get_document_schema(reference_url).await?
                                {
                                    (
                                        Cow::Owned(document_schema.schema_url),
                                        Cow::Owned(document_schema.definitions),
                                    )
                                } else {
                                    (schema_url, definitions)
                                }
                            }
                            None => (schema_url, definitions),
                        }
                    };

                    match value_schema {
                        ValueSchema::OneOf(OneOfSchema { schemas, .. })
                        | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                        | ValueSchema::AllOf(AllOfSchema { schemas, .. }) => {
                            for schema in schemas.write().await.iter_mut() {
                                schema
                                    .resolve(schema_url.clone(), definitions.clone(), schema_store)
                                    .await?;
                            }
                        }
                        _ => {}
                    }

                    Ok(Some(CurrentSchema {
                        value_schema: Cow::Borrowed(value_schema),
                        schema_url,
                        definitions,
                    }))
                }
            }
        })
    }
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
) -> Result<Option<ValueSchema>, crate::Error> {
    if !pointer.starts_with('#') {
        return Ok(None);
    }

    let path = &pointer[1..]; // Remove the leading '#'
    if path.is_empty() {
        return Ok(schema_node.as_object().and_then(ValueSchema::new));
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
        tombi_json::ValueNode::Object(obj) => Ok(ValueSchema::new(obj)),
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

            if hex_chars.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex_chars, 16) {
                    result.push(byte);
                    continue;
                }
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

    use crate::{schema::referable_schema::resolve_json_pointer, ValueSchema};

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
        let result = resolve_json_pointer(&value_node, "#/foo/bar%2Fbaz");
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

        let result = resolve_json_pointer(&value_node, "#/test/path%2Fwith%20spaces");
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
        let result = resolve_json_pointer(&value_node, "#/foo/bar%2");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let result = resolve_json_pointer(&value_node, "#/foo/baz%2G");
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
        let result = resolve_json_pointer(&value_node, "#/foo/bar~1baz");
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            assert!(matches!(schema, ValueSchema::String(_)));
        }

        let result = resolve_json_pointer(&value_node, "#/foo/qux~0tilde");
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            assert!(matches!(schema, ValueSchema::String(_)));
        }
    }
}
