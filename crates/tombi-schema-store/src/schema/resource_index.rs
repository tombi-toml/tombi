use std::str::FromStr;

use tombi_hashmap::HashMap;
use tombi_json::{ObjectNode, ValueNode};
use tombi_text::Range;

use crate::{JsonSchemaDialect, SchemaUri};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BuildError {
    InvalidId {
        id: String,
        base_uri: SchemaUri,
        range: Range,
    },
    IdWithNonEmptyFragment {
        id: String,
        range: Range,
    },
    DuplicateResource {
        uri: SchemaUri,
        first_range: Range,
        duplicate_range: Range,
    },
    InvalidAnchor {
        keyword: &'static str,
        anchor: String,
        range: Range,
    },
    DuplicateAnchor {
        resource_uri: SchemaUri,
        anchor: String,
        first_range: Range,
        duplicate_range: Range,
    },
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidId {
                id,
                base_uri,
                range,
            } => write!(f, "invalid $id {id:?} relative to {base_uri} at {range}"),
            Self::IdWithNonEmptyFragment { id, range } => {
                write!(
                    f,
                    "$id must not contain a non-empty fragment: {id:?} at {range}"
                )
            }
            Self::DuplicateResource {
                uri,
                first_range,
                duplicate_range,
            } => write!(
                f,
                "duplicate schema resource URI {uri}: first at {first_range}, duplicate at {duplicate_range}"
            ),
            Self::InvalidAnchor {
                keyword,
                anchor,
                range,
            } => write!(f, "invalid {keyword} name {anchor:?} at {range}"),
            Self::DuplicateAnchor {
                resource_uri,
                anchor,
                first_range,
                duplicate_range,
            } => write!(
                f,
                "duplicate anchor {anchor:?} in schema resource {resource_uri}: first at {first_range}, duplicate at {duplicate_range}"
            ),
        }
    }
}

impl std::error::Error for BuildError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceTarget {
    /// JSON Pointer from the root of the physical schema document.
    pub pointer: String,
    pub range: Range,
}

#[derive(Debug, Clone)]
pub(crate) struct ResourceMetadata {
    /// JSON Pointer from the root of the physical schema document.
    pub root_pointer: String,
    pub root_range: Range,
    pub dialect: Option<JsonSchemaDialect>,
    pub anchors: HashMap<String, ResourceTarget>,
    pub dynamic_anchors: HashMap<String, ResourceTarget>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResourceIndex {
    retrieval_uri: SchemaUri,
    root_resource_uri: SchemaUri,
    resources: HashMap<SchemaUri, ResourceMetadata>,
    effective_base_by_position: HashMap<tombi_text::Position, SchemaUri>,
}

impl ResourceIndex {
    pub fn build(
        root: &ValueNode,
        retrieval_uri: &SchemaUri,
        default_dialect: Option<JsonSchemaDialect>,
    ) -> Result<Self, BuildError> {
        let retrieval_uri = without_fragment(retrieval_uri.clone());
        let mut index = Self {
            root_resource_uri: retrieval_uri.clone(),
            retrieval_uri: retrieval_uri.clone(),
            resources: HashMap::default(),
            effective_base_by_position: HashMap::default(),
        };

        let ValueNode::Object(root_object) = root else {
            index.resources.insert(
                retrieval_uri.clone(),
                ResourceMetadata {
                    root_pointer: "#".to_owned(),
                    root_range: root.range(),
                    dialect: default_dialect,
                    anchors: HashMap::default(),
                    dynamic_anchors: HashMap::default(),
                },
            );
            return Ok(index);
        };

        let mut stack = vec![Frame {
            object: root_object,
            pointer: "#".to_owned(),
            incoming_base: retrieval_uri.clone(),
            resource_uri: retrieval_uri,
            dialect: default_dialect,
            is_root: true,
        }];

        while let Some(frame) = stack.pop() {
            let dialect = schema_dialect(frame.object).or(frame.dialect);
            let mut effective_base = frame.incoming_base;
            let mut resource_uri = frame.resource_uri;
            let mut starts_resource = frame.is_root;

            if let Some(id_node) = frame.object.get("$id")
                && let Some(id) = id_node.as_str()
            {
                effective_base = resolve_id(id, &effective_base, id_node.range())?;
                let has_non_empty_fragment =
                    effective_base.fragment().is_some_and(|f| !f.is_empty());
                if has_non_empty_fragment
                    && dialect.is_some_and(|dialect| dialect >= JsonSchemaDialect::Draft2019_09)
                {
                    return Err(BuildError::IdWithNonEmptyFragment {
                        id: id.to_owned(),
                        range: id_node.range(),
                    });
                }

                // Draft 7 permits legacy fragment-bearing identifiers. They change
                // the base URI but do not start an independently addressable resource.
                if !has_non_empty_fragment {
                    effective_base = without_fragment(effective_base);
                    resource_uri = effective_base.clone();
                    starts_resource = true;
                }
            }

            // ReferableSchema only asks for the base of an object containing a
            // reference. Keeping entries for every schema object made the
            // index proportional to the entire document for no benefit.
            if ["$ref", "$dynamicRef", "$recursiveRef"]
                .iter()
                .any(|keyword| frame.object.get(keyword).is_some())
            {
                index
                    .effective_base_by_position
                    .insert(frame.object.range.start, effective_base.clone());
            }

            if starts_resource {
                if !frame.is_root && resource_uri == index.retrieval_uri {
                    let first_range = index
                        .resources
                        .get(&index.root_resource_uri)
                        .expect("the root resource is indexed before its children")
                        .root_range;
                    return Err(BuildError::DuplicateResource {
                        uri: resource_uri,
                        first_range,
                        duplicate_range: frame.object.range,
                    });
                }
                let metadata = ResourceMetadata {
                    root_pointer: frame.pointer.clone(),
                    root_range: frame.object.range,
                    dialect,
                    anchors: HashMap::default(),
                    dynamic_anchors: HashMap::default(),
                };
                if let Some(previous) = index.resources.insert(resource_uri.clone(), metadata) {
                    return Err(BuildError::DuplicateResource {
                        uri: resource_uri,
                        first_range: previous.root_range,
                        duplicate_range: frame.object.range,
                    });
                }
                if frame.is_root {
                    index.root_resource_uri = resource_uri.clone();
                }
            }

            index.collect_anchors(frame.object, &frame.pointer, &resource_uri, dialect)?;

            let mut children = Vec::new();
            collect_schema_children(frame.object, dialect, &frame.pointer, &mut children);
            for (object, pointer) in children.into_iter().rev() {
                stack.push(Frame {
                    object,
                    pointer,
                    incoming_base: effective_base.clone(),
                    resource_uri: resource_uri.clone(),
                    dialect,
                    is_root: false,
                });
            }
        }

        Ok(index)
    }

    pub fn root_resource_uri(&self) -> &SchemaUri {
        &self.root_resource_uri
    }

    pub fn resources(&self) -> &HashMap<SchemaUri, ResourceMetadata> {
        &self.resources
    }

    pub fn resource(&self, uri: &SchemaUri) -> Option<&ResourceMetadata> {
        let uri = without_fragment(uri.clone());
        if uri == self.retrieval_uri {
            self.resources.get(&self.root_resource_uri)
        } else {
            self.resources.get(&uri)
        }
    }

    pub fn physical_fragment(&self, uri: &SchemaUri, fragment: &str) -> Option<String> {
        let resource = self.resource(uri)?;
        if fragment.starts_with('/') {
            return Some(format!(
                "{}{}",
                resource.root_pointer.strip_prefix('#').unwrap_or(""),
                fragment
            ));
        }

        let anchor = format!("#{fragment}");
        resource
            .anchors
            .get(&anchor)
            .or_else(|| resource.dynamic_anchors.get(&anchor))
            .map(|target| {
                target
                    .pointer
                    .strip_prefix('#')
                    .unwrap_or(&target.pointer)
                    .to_owned()
            })
    }

    pub fn effective_base(&self, position: tombi_text::Position) -> Option<&SchemaUri> {
        self.effective_base_by_position.get(&position)
    }

    fn collect_anchors(
        &mut self,
        object: &ObjectNode,
        pointer: &str,
        resource_uri: &SchemaUri,
        dialect: Option<JsonSchemaDialect>,
    ) -> Result<(), BuildError> {
        if crate::supports_keyword(dialect, "$anchor")
            && let Some(node) = object.get("$anchor")
            && let Some(anchor) = node.as_str()
        {
            self.insert_anchor(
                resource_uri,
                "$anchor",
                anchor,
                pointer,
                node.range(),
                false,
            )?;
        }

        if crate::supports_keyword(dialect, "$dynamicAnchor")
            && let Some(node) = object.get("$dynamicAnchor")
            && let Some(anchor) = node.as_str()
        {
            self.insert_anchor(
                resource_uri,
                "$dynamicAnchor",
                anchor,
                pointer,
                node.range(),
                true,
            )?;
        }

        if crate::supports_keyword(dialect, "$recursiveAnchor")
            && object.get("$recursiveAnchor").and_then(ValueNode::as_bool) == Some(true)
        {
            let range = object
                .get("$recursiveAnchor")
                .map(ValueNode::range)
                .unwrap_or(object.range);
            self.insert_named_target(resource_uri, "#", pointer, range, true)?;
        }

        Ok(())
    }

    fn insert_anchor(
        &mut self,
        resource_uri: &SchemaUri,
        keyword: &'static str,
        anchor: &str,
        pointer: &str,
        range: Range,
        dynamic: bool,
    ) -> Result<(), BuildError> {
        if !is_plain_name(anchor) {
            return Err(BuildError::InvalidAnchor {
                keyword,
                anchor: anchor.to_owned(),
                range,
            });
        }
        self.insert_named_target(resource_uri, &format!("#{anchor}"), pointer, range, dynamic)
    }

    fn insert_named_target(
        &mut self,
        resource_uri: &SchemaUri,
        name: &str,
        pointer: &str,
        range: Range,
        dynamic: bool,
    ) -> Result<(), BuildError> {
        let resource = self
            .resources
            .get_mut(resource_uri)
            .expect("the owning resource is inserted before its anchors");
        let previous = resource
            .anchors
            .get(name)
            .or_else(|| resource.dynamic_anchors.get(name));
        if let Some(previous) = previous {
            return Err(BuildError::DuplicateAnchor {
                resource_uri: resource_uri.clone(),
                anchor: name.to_owned(),
                first_range: previous.range,
                duplicate_range: range,
            });
        }

        let target = ResourceTarget {
            pointer: pointer.to_owned(),
            range,
        };
        if dynamic {
            resource.dynamic_anchors.insert(name.to_owned(), target);
        } else {
            resource.anchors.insert(name.to_owned(), target);
        }
        Ok(())
    }
}

struct Frame<'a> {
    object: &'a ObjectNode,
    pointer: String,
    incoming_base: SchemaUri,
    resource_uri: SchemaUri,
    dialect: Option<JsonSchemaDialect>,
    is_root: bool,
}

fn resolve_id(id: &str, base_uri: &SchemaUri, range: Range) -> Result<SchemaUri, BuildError> {
    base_uri
        .join(id)
        .map(SchemaUri::from)
        .or_else(|_| SchemaUri::from_str(id))
        .map_err(|_| BuildError::InvalidId {
            id: id.to_owned(),
            base_uri: base_uri.clone(),
            range,
        })
}

fn without_fragment(mut uri: SchemaUri) -> SchemaUri {
    uri.set_fragment(None);
    uri
}

fn schema_dialect(object: &ObjectNode) -> Option<JsonSchemaDialect> {
    object
        .get("$schema")
        .and_then(ValueNode::as_str)
        .and_then(|uri| JsonSchemaDialect::try_from(uri).ok())
}

fn is_plain_name(anchor: &str) -> bool {
    let mut chars = anchor.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn collect_schema_children<'a>(
    object: &'a ObjectNode,
    dialect: Option<JsonSchemaDialect>,
    pointer: &str,
    output: &mut Vec<(&'a ObjectNode, String)>,
) {
    const SINGLE_SCHEMA_KEYWORDS: &[&str] = &[
        "additionalProperties",
        "unevaluatedProperties",
        "unevaluatedItems",
        "propertyNames",
        "contains",
        "not",
        "if",
        "then",
        "else",
        "contentSchema",
        "additionalItems",
    ];
    const ARRAY_SCHEMA_KEYWORDS: &[&str] = &["allOf", "anyOf", "oneOf", "prefixItems"];
    const MAP_SCHEMA_KEYWORDS: &[&str] = &[
        "$defs",
        "definitions",
        "properties",
        "patternProperties",
        "dependentSchemas",
    ];

    for keyword in SINGLE_SCHEMA_KEYWORDS {
        if crate::supports_keyword(dialect, keyword)
            && let Some(child) = object.get(keyword).and_then(ValueNode::as_object)
        {
            output.push((child, child_pointer(pointer, keyword)));
        }
    }

    for keyword in ARRAY_SCHEMA_KEYWORDS {
        if crate::supports_keyword(dialect, keyword)
            && let Some(array) = object.get(keyword).and_then(ValueNode::as_array)
        {
            let base = child_pointer(pointer, keyword);
            for (index, child) in array.items.iter().enumerate() {
                if let Some(child) = child.as_object() {
                    output.push((child, format!("{base}/{index}")));
                }
            }
        }
    }

    for keyword in MAP_SCHEMA_KEYWORDS {
        if crate::supports_keyword(dialect, keyword)
            && let Some(map) = object.get(keyword).and_then(ValueNode::as_object)
        {
            let base = child_pointer(pointer, keyword);
            for (name, child) in &map.properties {
                if let Some(child) = child.as_object() {
                    output.push((
                        child,
                        format!("{base}/{}", escape_json_pointer_token(&name.value)),
                    ));
                }
            }
        }
    }

    if crate::supports_keyword(dialect, "items")
        && let Some(items) = object.get("items")
    {
        let base = child_pointer(pointer, "items");
        match items {
            ValueNode::Object(child) => output.push((child, base)),
            ValueNode::Array(array)
                if dialect.is_none_or(|dialect| dialect < JsonSchemaDialect::Draft2020_12) =>
            {
                for (index, child) in array.items.iter().enumerate() {
                    if let Some(child) = child.as_object() {
                        output.push((child, format!("{base}/{index}")));
                    }
                }
            }
            _ => {}
        }
    }

    if crate::supports_keyword(dialect, "dependencies")
        && let Some(map) = object.get("dependencies").and_then(ValueNode::as_object)
    {
        let base = child_pointer(pointer, "dependencies");
        for (name, child) in &map.properties {
            if let Some(child) = child.as_object() {
                output.push((
                    child,
                    format!("{base}/{}", escape_json_pointer_token(&name.value)),
                ));
            }
        }
    }
}

fn child_pointer(parent: &str, token: &str) -> String {
    format!("{parent}/{}", escape_json_pointer_token(token))
}

fn escape_json_pointer_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(value: &str) -> SchemaUri {
        SchemaUri::from_str(value).expect("valid test URI")
    }

    fn schema(value: &str) -> ValueNode {
        ValueNode::from_str(value).expect("valid test schema")
    }

    #[test]
    fn indexes_relative_compound_resources_and_effective_bases() {
        let root = schema(
            r#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": "https://example.com/root/bundle.json",
                "$defs": {
                    "first": {
                        "$id": "models/first",
                        "$defs": {
                            "nested": { "$id": "../nested", "type": "string" }
                        }
                    }
                }
            }"#,
        );
        let index = ResourceIndex::build(&root, &uri("file:///tmp/bundle.json"), None).unwrap();

        assert_eq!(
            index.root_resource_uri().as_str(),
            "https://example.com/root/bundle.json"
        );
        assert!(
            index
                .resource(&uri("https://example.com/root/models/first"))
                .is_some()
        );
        let nested = index
            .resource(&uri("https://example.com/root/nested"))
            .unwrap();
        assert_eq!(nested.root_pointer, "#/$defs/first/$defs/nested");
        assert_eq!(
            index.physical_fragment(&uri("https://example.com/root/nested"), "/type"),
            Some("/$defs/first/$defs/nested/type".to_owned())
        );
        assert_eq!(
            index
                .resource(&uri("file:///tmp/bundle.json"))
                .unwrap()
                .root_pointer,
            "#"
        );
    }

    #[test]
    fn scopes_equal_anchor_names_to_their_resources() {
        let root = schema(
            r#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": "https://example.com/bundle",
                "$anchor": "same",
                "$defs": {
                    "child": {
                        "$id": "child",
                        "$anchor": "same",
                        "$dynamicAnchor": "dynamic"
                    }
                }
            }"#,
        );
        let index = ResourceIndex::build(&root, &uri("file:///bundle.json"), None).unwrap();

        let root_resource = index.resource(index.root_resource_uri()).unwrap();
        let child = index.resource(&uri("https://example.com/child")).unwrap();
        assert_eq!(root_resource.anchors["#same"].pointer, "#");
        assert_eq!(child.anchors["#same"].pointer, "#/$defs/child");
        assert_eq!(child.dynamic_anchors["#dynamic"].pointer, "#/$defs/child");
        assert_eq!(
            index.physical_fragment(&uri("https://example.com/child"), "same"),
            Some("/$defs/child".to_owned())
        );
        assert_eq!(
            index.physical_fragment(&uri("https://example.com/child"), "dynamic"),
            Some("/$defs/child".to_owned())
        );
    }

    #[test]
    fn rejects_duplicate_canonical_resource_uri_after_normalization() {
        let root = schema(
            r#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": "https://EXAMPLE.com/a/bundle",
                "$defs": {
                    "a": { "$id": "../resource" },
                    "b": { "$id": "https://example.com/resource" }
                }
            }"#,
        );
        let error = ResourceIndex::build(&root, &uri("file:///bundle.json"), None).unwrap_err();
        assert!(matches!(error, BuildError::DuplicateResource { .. }));
    }

    #[test]
    fn reserves_retrieval_uri_for_the_root_resource() {
        let root = schema(
            r#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": "https://example.com/root",
                "$defs": {
                    "shadow": { "$id": "file:///tmp/bundle.json" }
                }
            }"#,
        );
        let root_range = root.range();
        let error = ResourceIndex::build(&root, &uri("file:///tmp/bundle.json"), None).unwrap_err();

        assert!(matches!(
            error,
            BuildError::DuplicateResource {
                uri: duplicate_uri,
                first_range,
                duplicate_range,
            } if duplicate_uri == uri("file:///tmp/bundle.json")
                && first_range == root_range
                && duplicate_range != root_range
        ));
    }

    #[test]
    fn rejects_duplicate_static_and_dynamic_anchor_names_in_one_resource() {
        let root = schema(
            r#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$anchor": "same",
                "$defs": {
                    "nested": { "$dynamicAnchor": "same" }
                }
            }"#,
        );
        let error = ResourceIndex::build(&root, &uri("file:///bundle.json"), None).unwrap_err();
        assert!(matches!(error, BuildError::DuplicateAnchor { .. }));

        let invalid = schema(
            r#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$anchor": "invalid:name"
            }"#,
        );
        let error = ResourceIndex::build(&invalid, &uri("file:///bundle.json"), None).unwrap_err();
        assert!(matches!(error, BuildError::InvalidAnchor { .. }));
    }

    #[test]
    fn ignores_id_lookalikes_in_instance_values() {
        let root = schema(
            r#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "const": { "$id": "https://example.com/not-a-schema" },
                "default": { "$id": "https://example.com/not-a-schema-either" },
                "properties": {
                    "actual": { "$id": "https://example.com/schema" }
                }
            }"#,
        );
        let index = ResourceIndex::build(&root, &uri("file:///bundle.json"), None).unwrap();

        assert_eq!(index.resources().len(), 2);
        assert!(index.resource(&uri("https://example.com/schema")).is_some());
        assert!(
            index
                .resource(&uri("https://example.com/not-a-schema"))
                .is_none()
        );
    }
}
