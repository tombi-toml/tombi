use std::{borrow::Cow, str::FromStr, sync::Arc};

use itertools::Itertools;
use tombi_schema_type::BoolDefaultTrue;
use tombi_x_keyword::StringFormat;

use crate::x_taplo::XTaplo;

use super::{
    AnchorCollector, Deprecation, DynamicAnchorCollector, SchemaDefinitions, SchemaMap, SchemaUri,
    SchemaView, bool_schema_view,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    Ref,
    DynamicRef,
    RecursiveRef,
}

#[derive(Debug, Clone)]
pub enum Referable<T> {
    Resolved {
        schema_base_uri: Option<SchemaUri>,
        value: Arc<T>,
        semantic_schema: Option<Arc<super::SemanticSchema>>,
    },
    Ref {
        reference: String,
        kind: ReferenceKind,
        semantic_schema: Option<Arc<super::SemanticSchema>>,
        title: Option<String>,
        description: Option<String>,
        default: Option<tombi_json::Value>,
        examples: Option<Vec<tombi_json::Value>>,
        deprecation: Option<Deprecation>,
    },
}

#[derive(Clone)]
pub struct CurrentSchema<'a> {
    pub schema_view: Arc<SchemaView>,
    /// Lossless JSON Schema representation and the source of truth.
    /// `schema_view` is only a derived, instance-specific presentation view.
    pub semantic_schema: Option<Arc<super::SemanticSchema>>,
    /// URI of this schema instance. Equals the physical document URI by default;
    /// fragment lookups may set a fragment-bearing URI (e.g. `file:///schema.json#/defs/Foo`).
    /// Distinct from [`Self::schema_base_uri`] (`$ref` base / `$id`) and
    /// [`Self::schema_document_uri`] (physical document).
    pub schema_uri: Cow<'a, SchemaUri>,
    /// **schema_base_uri**: Base for resolving `$ref` and relative references. Usually equals
    /// `schema_resource_uri`; may differ after resolving a root-level `$ref`.
    pub schema_base_uri: Cow<'a, SchemaUri>,
    /// **schema_document_uri**: Physical URI of the loaded JSON Schema document (file/http/etc).
    /// Not `$id`. Used for config lookup, cache, and opening the source file.
    pub schema_document_uri: Cow<'a, SchemaUri>,
    pub definitions: Cow<'a, SchemaDefinitions>,
    /// strict setting on root-schema/sub-schema level.
    pub strict: Option<BoolDefaultTrue>,
    /// Dynamic scope for `$dynamicRef` / `$recursiveRef`.
    /// Ordered innermost-first (`[0]` is the most recently entered schema resource).
    /// Matching anchors are searched outermost-first (see `resolve_dynamic_anchor_from_scope`).
    pub dynamic_scope: Vec<SchemaUri>,
}

impl<'a> CurrentSchema<'a> {
    pub fn into_owned(self) -> CurrentSchema<'static> {
        CurrentSchema {
            schema_view: self.schema_view,
            semantic_schema: self.semantic_schema,
            schema_uri: Cow::Owned(self.schema_uri.into_owned()),
            schema_base_uri: Cow::Owned(self.schema_base_uri.into_owned()),
            schema_document_uri: Cow::Owned(self.schema_document_uri.into_owned()),
            definitions: Cow::Owned(self.definitions.into_owned()),
            strict: self.strict,
            dynamic_scope: self.dynamic_scope,
        }
    }

    /// URI used to look up schema-associated format, lint, and override settings.
    pub fn schema_document_uri_for_config(&self) -> SchemaUri {
        let mut uri = self.schema_document_uri.clone().into_owned();
        if uri.fragment().is_some() {
            uri.set_fragment(None);
        }
        uri
    }

    /// Rebuilds this schema around a projected view, keeping the semantic
    /// schema as the source of truth. `None` keeps the current view, which is
    /// what a boolean `true` schema needs: it has no object payload to project,
    /// and its existing Anything view already represents the admitted instance.
    fn with_projected_view(
        &self,
        semantic_schema: &Arc<super::SemanticSchema>,
        projected_view: Option<SchemaView>,
    ) -> CurrentSchema<'static> {
        CurrentSchema {
            schema_view: projected_view
                .map(Arc::new)
                .unwrap_or_else(|| self.schema_view.clone()),
            semantic_schema: Some(semantic_schema.clone()),
            schema_uri: Cow::Owned(self.schema_uri.as_ref().clone()),
            schema_base_uri: Cow::Owned(self.schema_base_uri.as_ref().clone()),
            schema_document_uri: Cow::Owned(self.schema_document_uri.as_ref().clone()),
            definitions: Cow::Owned(self.definitions.as_ref().clone()),
            strict: self.strict,
            dynamic_scope: self.dynamic_scope.clone(),
        }
    }

    /// Projects this schema for the concrete instance type being handled.
    /// Keywords for other instance types remain semantically inert.
    pub fn for_instance_type(
        &self,
        instance_type: super::SchemaType,
        string_formats: Option<&[StringFormat]>,
    ) -> Option<CurrentSchema<'static>> {
        let semantic_schema = self.semantic_schema.as_ref()?;
        if !semantic_schema.accepts_instance_type(instance_type) {
            return None;
        }
        if self.schema_view.matches_instance_type(instance_type)
            && !self.requires_instance_projection(instance_type)
        {
            return Some(self.clone().into_owned());
        }
        Some(self.with_projected_view(
            semantic_schema,
            semantic_schema.schema_view_for_type(instance_type, string_formats),
        ))
    }

    /// Returns whether the current view omits constraints for this instance type.
    /// A compatible `$ref` sibling `type` is already represented by the resolved view.
    pub fn requires_instance_projection(&self, instance_type: super::SchemaType) -> bool {
        self.semantic_schema.as_deref().is_some_and(|semantic| {
            semantic.accepts_instance_type(instance_type)
                && (!self.schema_view.matches_instance_type(instance_type)
                    || semantic.root_reference_requires_instance_projection(instance_type))
        })
    }

    pub fn has_reference_projection_siblings(&self, instance_type: super::SchemaType) -> bool {
        self.semantic_schema
            .as_deref()
            .is_some_and(|semantic| semantic.root_reference_has_projection_siblings(instance_type))
    }

    pub fn for_completion(
        &self,
        string_formats: Option<&[StringFormat]>,
    ) -> Option<CurrentSchema<'static>> {
        let semantic_schema = self.semantic_schema.as_ref()?;
        if semantic_schema.has_references() {
            return Some(self.clone().into_owned());
        }
        Some(self.with_projected_view(
            semantic_schema,
            semantic_schema.completion_projection(string_formats),
        ))
    }
}

impl std::fmt::Debug for CurrentSchema<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CurrentSchema")
            .field("schema_view", &self.schema_view)
            .field("schema_uri", &self.schema_uri.to_string())
            .field("schema_base_uri", &self.schema_base_uri.to_string())
            .field("schema_document_uri", &self.schema_document_uri.to_string())
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

impl Referable<SchemaView> {
    pub fn new(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
        dialect: Option<crate::JsonSchemaDialect>,
        anchor_collector: Option<&mut AnchorCollector>,
        dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
    ) -> Option<Self> {
        let mut anchor_collector = anchor_collector;
        let mut dynamic_anchor_collector = dynamic_anchor_collector;
        if let Some(x_taplo) = object.get("x-taplo")
            && let Ok(x_taplo) = tombi_json::from_value_node::<XTaplo>(x_taplo.to_owned())
            && x_taplo.hidden == Some(true)
        {
            return None;
        }
        let (reference_kind, reference_value) = match (
            object.get("$ref").and_then(|v| v.as_str()),
            dialect
                .filter(|dialect| crate::supports_keyword(Some(*dialect), "$dynamicRef"))
                .and_then(|_| object.get("$dynamicRef").and_then(|v| v.as_str())),
            dialect
                .filter(|dialect| crate::supports_keyword(Some(*dialect), "$recursiveRef"))
                .and_then(|_| object.get("$recursiveRef").and_then(|v| v.as_str())),
        ) {
            (Some(reference), _, _) => (Some(ReferenceKind::Ref), Some(reference)),
            (None, Some(reference), _) => (Some(ReferenceKind::DynamicRef), Some(reference)),
            (None, None, Some(reference)) => {
                if reference == "#" {
                    (Some(ReferenceKind::RecursiveRef), Some(reference))
                } else {
                    (None, None)
                }
            }
            (None, None, None) => (None, None),
        };
        let referable = if let (Some(kind), Some(reference)) = (reference_kind, reference_value) {
            // Draft 7 treats a `$ref` object as a reference only. Newer drafts allow
            // assertion and applicator siblings to participate in evaluation.
            let supports_ref_siblings = dialect != Some(crate::JsonSchemaDialect::Draft07);
            let semantic_schema = supports_ref_siblings
                .then(|| Arc::new(super::SemanticSchema::from_object_node(object, dialect)))
                .filter(|schema| has_reference_projection_siblings(schema));
            Some(Referable::Ref {
                reference: reference.to_string(),
                kind,
                semantic_schema,
                // Keep annotation siblings for compatibility with existing Draft 7
                // schemas. They do not affect validation, unlike assertion and
                // applicator siblings, which remain disabled above.
                title: object
                    .get("title")
                    .and_then(|title| title.as_str().map(ToString::to_string)),
                description: object
                    .get("description")
                    .and_then(|description| description.as_str().map(ToString::to_string)),
                default: object.get("default").cloned().map(Into::into),
                examples: object
                    .get("examples")
                    .and_then(|examples| examples.as_array())
                    .map(|array| array.items.iter().map(Into::into).collect()),
                deprecation: Deprecation::new(object),
            })
        } else {
            let semantic_schema = Some(Arc::new(super::SemanticSchema::from_object_node(
                object, dialect,
            )));
            let schema_view = if semantic_schema.as_deref().is_some_and(|schema| {
                schema.has_type_assertion() || schema.has_direct_literal_assertion()
            }) {
                semantic_schema.as_ref().and_then(|semantic_schema| {
                    semantic_schema.completion_projection_with_collectors(
                        string_formats,
                        anchor_collector.as_deref_mut(),
                        dynamic_anchor_collector.as_deref_mut(),
                    )
                })
            } else if object.get("oneOf").is_some() {
                Some(SchemaView::OneOf(super::OneOfSchema::new(
                    object,
                    string_formats,
                    dialect,
                    anchor_collector.as_deref_mut(),
                    dynamic_anchor_collector.as_deref_mut(),
                )))
            } else if object.get("anyOf").is_some() {
                Some(SchemaView::AnyOf(super::AnyOfSchema::new(
                    object,
                    string_formats,
                    dialect,
                    anchor_collector.as_deref_mut(),
                    dynamic_anchor_collector.as_deref_mut(),
                )))
            } else if object.get("allOf").is_some() {
                Some(SchemaView::AllOf(super::AllOfSchema::new(
                    object,
                    string_formats,
                    dialect,
                    anchor_collector.as_deref_mut(),
                    dynamic_anchor_collector.as_deref_mut(),
                )))
            } else {
                Some(SchemaView::Anything(super::AnythingSchema {
                    title: object
                        .get("title")
                        .and_then(|value| value.as_str().map(ToOwned::to_owned)),
                    description: object
                        .get("description")
                        .and_then(|value| value.as_str().map(ToOwned::to_owned)),
                    range: object.range,
                }))
            };
            schema_view.map(|schema_view| Referable::Resolved {
                schema_base_uri: None,
                value: Arc::new(schema_view),
                semantic_schema,
            })
        };

        if let Some(referable) = referable.as_ref() {
            super::update_named_anchors(
                object,
                referable,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            );
            Self::collect_nested_definition_anchors(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            );
        }

        referable
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self, Referable::Resolved { .. })
    }

    fn collect_nested_definition_anchors(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
        dialect: Option<crate::JsonSchemaDialect>,
        mut anchor_collector: Option<&mut AnchorCollector>,
        mut dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
    ) {
        for defs_key in ["definitions", "$defs"] {
            let Some(tombi_json::ValueNode::Object(definitions)) = object.get(defs_key) else {
                continue;
            };

            for (_, value) in &definitions.properties {
                let starts_new_resource = value
                    .as_object()
                    .and_then(|definition| definition.get("$id"))
                    .and_then(tombi_json::ValueNode::as_str)
                    .is_some_and(|id| id.split_once('#').map_or(true, |(base, _)| !base.is_empty()));
                if starts_new_resource {
                    continue;
                }

                let _ = super::referable_from_schema_value(
                    value,
                    string_formats,
                    dialect,
                    anchor_collector.as_deref_mut(),
                    dynamic_anchor_collector.as_deref_mut(),
                );
            }
        }
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Referable::Ref { .. })
    }

    pub fn deprecation<'a: 'b, 'b>(
        &'a self,
    ) -> tombi_future::BoxFuture<'b, Option<crate::Deprecation>> {
        Box::pin(async move {
            match self {
                Referable::Resolved { value, .. } => value.deprecation().await,
                Referable::Ref { .. } => None,
            }
        })
    }

    pub async fn value_type(&self) -> crate::ValueType {
        match self {
            Referable::Resolved { value, .. } => value.value_type().await,
            Referable::Ref {
                reference, kind, ..
            } => {
                let ref_keyword = match kind {
                    ReferenceKind::Ref => "$ref",
                    ReferenceKind::DynamicRef => "$dynamicRef",
                    ReferenceKind::RecursiveRef => "$recursiveRef",
                };
                log::warn!(
                    "unresolved {ref_keyword} while determining value type: reference={reference}",
                );
                // Unknown under the current API surface (no schema context here).
                crate::ValueType::AnyOf(Vec::new())
            }
        }
    }

    pub fn resolve<'a: 'b, 'b>(
        &'a mut self,
        schema_base_uri: Cow<'a, SchemaUri>,
        definitions: Cow<'a, SchemaDefinitions>,
        strict: Option<BoolDefaultTrue>,
        schema_store: &'a crate::SchemaStore,
    ) -> tombi_future::BoxFuture<'b, Result<Option<CurrentSchema<'a>>, crate::Error>> {
        self.resolve_in_scope(schema_base_uri, definitions, strict, schema_store, None)
    }

    pub(crate) fn resolve_in_scope<'a: 'b, 'b>(
        &'a mut self,
        schema_base_uri: Cow<'a, SchemaUri>,
        definitions: Cow<'a, SchemaDefinitions>,
        strict: Option<BoolDefaultTrue>,
        schema_store: &'a crate::SchemaStore,
        parent_dynamic_scope: Option<&[SchemaUri]>,
    ) -> tombi_future::BoxFuture<'b, Result<Option<CurrentSchema<'a>>, crate::Error>> {
        let dynamic_scope = extend_dynamic_scope(
            parent_dynamic_scope.unwrap_or(&[]),
            schema_base_uri.as_ref(),
        );
        self.resolve_with_dynamic_scope(
            schema_base_uri,
            definitions,
            strict,
            schema_store,
            dynamic_scope,
        )
    }

    fn resolve_with_dynamic_scope<'a: 'b, 'b>(
        &'a mut self,
        schema_base_uri: Cow<'a, SchemaUri>,
        definitions: Cow<'a, SchemaDefinitions>,
        strict: Option<BoolDefaultTrue>,
        schema_store: &'a crate::SchemaStore,
        dynamic_scope: Vec<SchemaUri>,
    ) -> tombi_future::BoxFuture<'b, Result<Option<CurrentSchema<'a>>, crate::Error>> {
        Box::pin(async move {
            match self {
                Referable::Ref {
                    reference,
                    kind,
                    semantic_schema: ref_semantic_schema,
                    title,
                    description,
                    default,
                    examples,
                    deprecation,
                } => {
                    let dynamic_target = match kind {
                        ReferenceKind::DynamicRef => {
                            resolve_initial_dynamic_anchor_target(
                                reference,
                                schema_base_uri.as_ref(),
                                schema_store,
                            )
                            .await?
                        }
                        // `$recursiveRef` needs a different algorithm (initial `$ref` target
                        // bookending); handled below, not via named dynamic-anchor lookup.
                        ReferenceKind::RecursiveRef | ReferenceKind::Ref => None,
                    };
                    if *kind == ReferenceKind::RecursiveRef
                        && let Some(resolved) = resolve_recursive_ref(
                            reference,
                            schema_base_uri.as_ref(),
                            strict,
                            schema_store,
                            &dynamic_scope,
                        )
                        .await?
                    {
                        return Ok(Some(resolved));
                    }
                    let should_cache_resolution = *kind == ReferenceKind::Ref;
                    if let Some((base_schema_uri, dynamic_anchor_ref)) = dynamic_target {
                        let mut scope_for_dynamic_ref = dynamic_scope.clone();
                        if let Some(base_schema_uri) = base_schema_uri {
                            scope_for_dynamic_ref.insert(0, base_schema_uri);
                        }
                        if let Some((
                            mut referable_schema,
                            owner_schema_base_uri,
                            owner_definitions,
                        )) = resolve_dynamic_anchor_from_scope(
                            &dynamic_anchor_ref,
                            &scope_for_dynamic_ref,
                            schema_store,
                        )
                        .await?
                        {
                            apply_ref_semantics(&mut referable_schema, ref_semantic_schema.clone());
                            apply_ref_annotations(
                                &mut referable_schema,
                                title.as_ref(),
                                description.as_ref(),
                                default.as_ref(),
                                examples.as_ref(),
                                deprecation.clone(),
                            );
                            if should_cache_resolution {
                                *self = referable_schema;
                                return self
                                    .resolve_with_dynamic_scope(
                                        Cow::Owned(owner_schema_base_uri),
                                        Cow::Owned(owner_definitions),
                                        strict,
                                        schema_store,
                                        scope_for_dynamic_ref,
                                    )
                                    .await;
                            }
                            return referable_schema
                                .resolve_with_dynamic_scope(
                                    Cow::Owned(owner_schema_base_uri),
                                    Cow::Owned(owner_definitions),
                                    strict,
                                    schema_store,
                                    scope_for_dynamic_ref,
                                )
                                .await
                                .map(|resolved| resolved.map(CurrentSchema::into_owned));
                        }
                    }

                    let definition_schema =
                        { resolve_from_schema_map(&definitions, reference).await };
                    let anchor_schema = if definition_schema.is_none() {
                        resolve_anchor_reference(reference, &schema_base_uri, schema_store).await?
                    } else {
                        None
                    };
                    if let Some(mut referable_schema) = definition_schema.or(anchor_schema) {
                        apply_ref_semantics(&mut referable_schema, ref_semantic_schema.clone());
                        apply_ref_annotations(
                            &mut referable_schema,
                            title.as_ref(),
                            description.as_ref(),
                            default.as_ref(),
                            examples.as_ref(),
                            deprecation.clone(),
                        );

                        if should_cache_resolution {
                            *self = referable_schema;
                        } else {
                            return referable_schema
                                .resolve_with_dynamic_scope(
                                    schema_base_uri,
                                    definitions,
                                    strict,
                                    schema_store,
                                    dynamic_scope,
                                )
                                .await
                                .map(|resolved| resolved.map(CurrentSchema::into_owned));
                        }
                    } else if is_json_pointer(reference) {
                        let pointer = reference;
                        let Some(mut resolved) = resolve_pointer_current_schema(
                            schema_store,
                            schema_base_uri.as_ref(),
                            pointer,
                            schema_store
                                .schema_resource_dialect(schema_base_uri.as_ref())
                                .await,
                            &definitions,
                            strict,
                            &dynamic_scope,
                        )
                        .await?
                        else {
                            return Ok(None);
                        };
                        if title.is_some() || description.is_some() {
                            let schema_view = Arc::make_mut(&mut resolved.schema_view);
                            schema_view.set_title(title.to_owned());
                            schema_view.set_description(description.to_owned());
                        }
                        if let Some(default) = default {
                            let schema_view = Arc::make_mut(&mut resolved.schema_view);
                            schema_view.set_default(Some(default.clone()));
                        }
                        if let Some(examples) = examples {
                            let schema_view = Arc::make_mut(&mut resolved.schema_view);
                            schema_view.set_examples(Some(examples.clone()));
                        }
                        if let Some(deprecation) = deprecation {
                            let schema_view = Arc::make_mut(&mut resolved.schema_view);
                            schema_view.set_deprecation(deprecation.clone());
                        }
                        if let Some(target) = resolved.semantic_schema.clone() {
                            resolved.semantic_schema =
                                combine_ref_semantics(ref_semantic_schema.clone(), Some(target));
                        }
                        return Ok(Some(resolved));
                    } else if let Some(resolved_reference) = resolve_external_reference(
                        reference,
                        schema_base_uri.as_ref(),
                        strict,
                        schema_store,
                        &dynamic_scope,
                    )
                    .await?
                    {
                        if ref_semantic_schema
                            .as_deref()
                            .is_some_and(has_reference_projection_siblings)
                        {
                            let source_schema_uri = schema_base_uri.as_ref().clone();
                            let source_definitions = definitions.clone().into_owned();
                            let local_semantic = ref_semantic_schema
                                .clone()
                                .expect("reference siblings have semantic schema");
                            let range = local_semantic.range();
                            let schemas = vec![
                                Referable::Resolved {
                                    schema_base_uri: Some(source_schema_uri),
                                    value: Arc::new(SchemaView::Anything(super::AnythingSchema {
                                        title: None,
                                        description: None,
                                        range,
                                    })),
                                    semantic_schema: Some(local_semantic),
                                },
                                Referable::Resolved {
                                    schema_base_uri: Some(
                                        resolved_reference.schema_base_uri.as_ref().clone(),
                                    ),
                                    value: resolved_reference.schema_view.clone(),
                                    semantic_schema: resolved_reference.semantic_schema.clone(),
                                },
                            ];
                            let resolved_referable = Referable::Resolved {
                                schema_base_uri: None,
                                value: Arc::new(SchemaView::AllOf(super::AllOfSchema {
                                    title: title.clone(),
                                    description: description.clone(),
                                    range,
                                    schemas: Arc::new(tokio::sync::RwLock::new(schemas)),
                                    default: default.clone(),
                                    examples: examples.clone(),
                                    deprecation: deprecation.clone(),
                                    reference_siblings: true,
                                    ..Default::default()
                                })),
                                semantic_schema: None,
                            };
                            let should_cache_resolution = should_cache_resolution
                                && resolved_reference.schema_uri.as_ref()
                                    == resolved_reference.schema_base_uri.as_ref();
                            if should_cache_resolution {
                                *self = resolved_referable;
                                return self
                                    .resolve_with_dynamic_scope(
                                        schema_base_uri,
                                        Cow::Owned(source_definitions),
                                        strict,
                                        schema_store,
                                        dynamic_scope,
                                    )
                                    .await;
                            }
                            let mut resolved_referable = resolved_referable;
                            return resolved_referable
                                .resolve_with_dynamic_scope(
                                    schema_base_uri,
                                    Cow::Owned(source_definitions),
                                    strict,
                                    schema_store,
                                    dynamic_scope,
                                )
                                .await
                                .map(|resolved| resolved.map(CurrentSchema::into_owned));
                        }

                        let mut resolved_value = resolved_reference.schema_view.clone();
                        if title.is_some() || description.is_some() {
                            let schema_view = Arc::make_mut(&mut resolved_value);
                            schema_view.set_title(title.to_owned());
                            schema_view.set_description(description.to_owned());
                        }
                        if let Some(default) = default {
                            let schema_view = Arc::make_mut(&mut resolved_value);
                            schema_view.set_default(Some(default.clone()));
                        }
                        if let Some(examples) = examples {
                            let schema_view = Arc::make_mut(&mut resolved_value);
                            schema_view.set_examples(Some(examples.clone()));
                        }
                        if let Some(deprecation) = deprecation {
                            let schema_view = Arc::make_mut(&mut resolved_value);
                            schema_view.set_deprecation(deprecation.clone());
                        }

                        let resolved_referable = Referable::Resolved {
                            schema_base_uri: Some(
                                resolved_reference.schema_base_uri.as_ref().clone(),
                            ),
                            value: resolved_value,
                            semantic_schema: combine_ref_semantics(
                                ref_semantic_schema.clone(),
                                resolved_reference.semantic_schema.clone(),
                            ),
                        };
                        let dynamic_scope = resolved_reference.dynamic_scope.clone();
                        let should_cache_resolution = should_cache_resolution
                            && resolved_reference.schema_uri.as_ref()
                                == resolved_reference.schema_base_uri.as_ref();
                        if should_cache_resolution {
                            *self = resolved_referable;
                            return self
                                .resolve_with_dynamic_scope(
                                    Cow::Owned(resolved_reference.schema_base_uri.into_owned()),
                                    Cow::Owned(resolved_reference.definitions.into_owned()),
                                    resolved_reference.strict,
                                    schema_store,
                                    dynamic_scope,
                                )
                                .await;
                        }
                        let mut resolved_referable = resolved_referable;
                        return resolved_referable
                            .resolve_with_dynamic_scope(
                                Cow::Owned(resolved_reference.schema_base_uri.into_owned()),
                                Cow::Owned(resolved_reference.definitions.into_owned()),
                                resolved_reference.strict,
                                schema_store,
                                dynamic_scope,
                            )
                            .await
                            .map(|resolved| resolved.map(CurrentSchema::into_owned));
                    } else {
                        return Err(crate::Error::UnsupportedReference {
                            reference: reference.to_owned(),
                            schema_uri: schema_base_uri.as_ref().to_owned(),
                        });
                    }

                    self.resolve_with_dynamic_scope(
                        schema_base_uri,
                        definitions,
                        strict,
                        schema_store,
                        dynamic_scope,
                    )
                    .await
                }
                Referable::Resolved {
                    schema_base_uri: reference_url,
                    value: schema_view,
                    semantic_schema,
                } => {
                    let (schema_uri, schema_base_uri, schema_document_uri, definitions) = {
                        match reference_url {
                            Some(reference_url) => {
                                if let Some(document_schema) =
                                    schema_store.try_get_document_schema(reference_url).await?
                                {
                                    (
                                        Cow::Owned(document_schema.schema_uri.clone()),
                                        Cow::Owned(document_schema.schema_base_uri().clone()),
                                        Cow::Owned(document_schema.schema_document_uri().clone()),
                                        Cow::Owned(document_schema.definitions.clone()),
                                    )
                                } else {
                                    let schema_document_uri = Cow::Owned(
                                        schema_store
                                            .schema_document_uri_for(schema_base_uri.as_ref())
                                            .await,
                                    );
                                    (
                                        Cow::Owned(reference_url.clone()),
                                        schema_base_uri,
                                        schema_document_uri,
                                        definitions,
                                    )
                                }
                            }
                            None => {
                                let schema_document_uri = schema_store
                                    .schema_document_uri_for(schema_base_uri.as_ref())
                                    .await;
                                (
                                    Cow::Owned(schema_document_uri.clone()),
                                    schema_base_uri,
                                    Cow::Owned(schema_document_uri),
                                    definitions,
                                )
                            }
                        }
                    };

                    Ok(Some(CurrentSchema {
                        schema_view: schema_view.clone(),
                        semantic_schema: semantic_schema.clone(),
                        schema_uri,
                        schema_base_uri,
                        schema_document_uri,
                        definitions,
                        strict,
                        dynamic_scope,
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
        schema_base_uri: Cow<'_, SchemaUri>,
        definitions: Cow<'_, SchemaDefinitions>,
        strict: Option<BoolDefaultTrue>,
        schema_store: &crate::SchemaStore,
        parent_dynamic_scope: Option<&[SchemaUri]>,
    ) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
        match self {
            Referable::Ref { .. } => Ok(None),
            Referable::Resolved {
                schema_base_uri: reference_url,
                value: schema_view,
                semantic_schema,
            } => {
                let (
                    schema_uri,
                    resolved_schema_base_uri,
                    schema_document_uri,
                    definitions,
                    dynamic_scope,
                ) =
                    match reference_url {
                        Some(reference_url) => {
                            if let Some(document_schema) =
                                schema_store.try_get_document_schema(reference_url).await?
                            {
                                (
                                    document_schema.schema_uri.clone(),
                                    document_schema.schema_base_uri().clone(),
                                    document_schema.schema_document_uri().clone(),
                                    document_schema.definitions.clone(),
                                    document_schema
                                        .dynamic_scope(parent_dynamic_scope.unwrap_or(&[])),
                                )
                            } else {
                                (
                                    reference_url.clone(),
                                    schema_base_uri.clone().into_owned(),
                                    schema_store
                                        .schema_document_uri_for(schema_base_uri.as_ref())
                                        .await,
                                    definitions.into_owned(),
                                    extend_dynamic_scope(
                                        parent_dynamic_scope.unwrap_or(&[]),
                                        schema_base_uri.as_ref(),
                                    ),
                                )
                            }
                        }
                        None => {
                            let schema_document_uri = schema_store
                                .schema_document_uri_for(schema_base_uri.as_ref())
                                .await;
                            (
                                schema_document_uri.clone(),
                                schema_base_uri.clone().into_owned(),
                                schema_document_uri,
                                definitions.into_owned(),
                                extend_dynamic_scope(
                                    parent_dynamic_scope.unwrap_or(&[]),
                                    schema_base_uri.as_ref(),
                                ),
                            )
                        }
                    };

                Ok(Some(CurrentSchema {
                    schema_view: schema_view.clone(),
                    semantic_schema: semantic_schema.clone(),
                    schema_uri: Cow::Owned(schema_uri),
                    schema_document_uri: Cow::Owned(schema_document_uri),
                    definitions: Cow::Owned(definitions),
                    strict,
                    dynamic_scope,
                    schema_base_uri: Cow::Owned(resolved_schema_base_uri),
                }))
            }
        }
    }
}

fn apply_ref_annotations(
    referable_schema: &mut Referable<SchemaView>,
    title: Option<&String>,
    description: Option<&String>,
    default: Option<&tombi_json::Value>,
    examples: Option<&Vec<tombi_json::Value>>,
    deprecation: Option<Deprecation>,
) {
    match referable_schema {
        Referable::Resolved {
            value: schema_view, ..
        } => {
            let schema_view = Arc::make_mut(schema_view);
            if let Some(title) = title {
                schema_view.set_title(Some(title.clone()));
            }
            if let Some(description) = description {
                schema_view.set_description(Some(description.clone()));
            }
            if let Some(default) = default {
                schema_view.set_default(Some(default.clone()));
            }
            if let Some(examples) = examples {
                schema_view.set_examples(Some(examples.clone()));
            }
            if let Some(deprecation) = deprecation {
                schema_view.set_deprecation(deprecation);
            }
        }
        Referable::Ref {
            title: ref_title,
            description: ref_description,
            default: ref_default,
            examples: ref_examples,
            deprecation: ref_deprecation,
            ..
        } => {
            if let Some(title) = title {
                *ref_title = Some(title.clone());
            }
            if let Some(description) = description {
                *ref_description = Some(description.clone());
            }
            if let Some(default) = default {
                *ref_default = Some(default.clone());
            }
            if let Some(examples) = examples {
                *ref_examples = Some(examples.clone());
            }
            if let Some(deprecation) = deprecation {
                *ref_deprecation = Some(deprecation);
            }
        }
    }
}

fn combine_ref_semantics(
    local: Option<Arc<super::SemanticSchema>>,
    target: Option<Arc<super::SemanticSchema>>,
) -> Option<Arc<super::SemanticSchema>> {
    match (local, target) {
        (Some(local), Some(target)) => Some(Arc::new(super::SemanticSchema::composite(
            super::SemanticCompositeKind::Reference,
            vec![local.as_ref().clone(), target.as_ref().clone()],
            local.range(),
        ))),
        (Some(schema), None) | (None, Some(schema)) => Some(schema),
        (None, None) => None,
    }
}

fn has_reference_projection_siblings(schema: &super::SemanticSchema) -> bool {
    use super::SchemaType;

    [
        SchemaType::Null,
        SchemaType::Boolean,
        SchemaType::Object,
        SchemaType::Array,
        SchemaType::Number,
        SchemaType::String,
        SchemaType::Integer,
    ]
    .into_iter()
    .any(|instance_type| schema.root_reference_has_projection_siblings(instance_type))
}

fn apply_ref_semantics(
    referable_schema: &mut Referable<SchemaView>,
    local: Option<Arc<super::SemanticSchema>>,
) {
    match referable_schema {
        Referable::Resolved {
            semantic_schema, ..
        }
        | Referable::Ref {
            semantic_schema, ..
        } => {
            *semantic_schema = combine_ref_semantics(local, semantic_schema.clone());
        }
    }
}

async fn resolve_from_schema_map(
    map: &std::sync::Arc<tokio::sync::RwLock<SchemaMap>>,
    reference: &str,
) -> Option<Referable<SchemaView>> {
    let map_guard = map.read().await;
    map_guard.get(reference).cloned()
}

async fn resolve_anchor_reference(
    reference: &str,
    schema_base_uri: &SchemaUri,
    schema_store: &crate::SchemaStore,
) -> Result<Option<Referable<SchemaView>>, crate::Error> {
    if !is_plain_name_anchor_reference(reference) {
        return Ok(None);
    }
    let Some(document_schema) = schema_store
        .try_get_document_schema(schema_base_uri)
        .await?
    else {
        return Ok(None);
    };
    Ok(resolve_from_schema_map(&document_schema.anchors, reference).await)
}

async fn resolve_dynamic_anchor_from_scope(
    reference: &str,
    dynamic_scope: &[SchemaUri],
    schema_store: &crate::SchemaStore,
) -> Result<Option<(Referable<SchemaView>, SchemaUri, SchemaDefinitions)>, crate::Error> {
    // Spec: choose the outermost (earliest) matching dynamic/recursive anchor.
    // `dynamic_scope` is stored innermost-first, so iterate in reverse.
    for scope_schema_uri in dynamic_scope.iter().rev() {
        let Some(document_schema) = schema_store
            .try_get_document_schema(scope_schema_uri)
            .await?
        else {
            continue;
        };
        let dynamic_anchors = &document_schema.dynamic_anchors;
        let dynamic_anchor_schema = {
            let anchors = dynamic_anchors.read().await;
            anchors.get(reference).cloned()
        };
        if let Some(dynamic_anchor_schema) = dynamic_anchor_schema {
            return Ok(Some((
                dynamic_anchor_schema,
                document_schema.schema_base_uri().clone(),
                document_schema.definitions.clone(),
            )));
        }
    }

    Ok(None)
}

/// Resolve `$recursiveRef` per draft-2019-09:
/// 1. Resolve against the current base URI as `$ref` would.
/// 2. Only if that target resource has `$recursiveAnchor: true`, search the
///    dynamic scope for the outermost resource that also has `$recursiveAnchor: true`.
/// 3. Otherwise behave identically to `$ref`.
async fn resolve_recursive_ref(
    reference: &str,
    schema_base_uri: &SchemaUri,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    dynamic_scope: &[SchemaUri],
) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
    if reference != "#" {
        return Ok(None);
    }

    // Caller already extended `dynamic_scope` with the current base URI.
    let Some(initial) = schema_store
        .try_get_document_schema(schema_base_uri)
        .await?
    else {
        return Ok(None);
    };
    let initial_has_anchor = {
        let anchors = initial.dynamic_anchors.read().await;
        anchors.contains_key("#")
    };
    let document_schema = if initial_has_anchor {
        outermost_recursive_anchor_document(dynamic_scope, schema_store)
            .await?
            .unwrap_or(initial)
    } else {
        initial
    };
    let Some(schema_view) = document_schema.schema_view.clone() else {
        return Ok(None);
    };

    let schema_base_uri = document_schema.schema_base_uri().clone();
    Ok(Some(CurrentSchema {
        schema_view,
        semantic_schema: document_schema.semantic_schema.clone(),
        schema_uri: Cow::Owned(document_schema.schema_uri.clone()),
        schema_document_uri: Cow::Owned(document_schema.schema_document_uri().clone()),
        definitions: Cow::Owned(document_schema.definitions.clone()),
        strict,
        dynamic_scope: extend_dynamic_scope(dynamic_scope, &schema_base_uri),
        schema_base_uri: Cow::Owned(schema_base_uri),
    }))
}

async fn outermost_recursive_anchor_document(
    dynamic_scope: &[SchemaUri],
    schema_store: &crate::SchemaStore,
) -> Result<Option<std::sync::Arc<super::DocumentSchema>>, crate::Error> {
    for scope_schema_uri in dynamic_scope.iter().rev() {
        let Some(document_schema) = schema_store
            .try_get_document_schema(scope_schema_uri)
            .await?
        else {
            continue;
        };
        let has_anchor = {
            let anchors = document_schema.dynamic_anchors.read().await;
            anchors.contains_key("#")
        };
        if has_anchor {
            return Ok(Some(document_schema));
        }
    }
    Ok(None)
}

async fn resolve_external_reference(
    reference: &str,
    base_schema_uri: &SchemaUri,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    parent_dynamic_scope: &[SchemaUri],
) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
    let joined = if let Ok(url) = base_schema_uri.join(reference) {
        Some(SchemaUri::from(url))
    } else {
        SchemaUri::from_str(reference).ok()
    };
    let Some(mut resolved_schema_uri) = joined else {
        return Ok(None);
    };

    let fragment = resolved_schema_uri
        .fragment()
        .map(ToString::to_string)
        .and_then(|fragment| (!fragment.is_empty()).then_some(fragment));
    resolved_schema_uri.set_fragment(None);

    let Some(document_schema) = schema_store
        .try_get_document_schema(&resolved_schema_uri)
        .await?
    else {
        return Ok(None);
    };

    let Some(fragment) = fragment else {
        let Some(schema_view) = document_schema.schema_view.as_ref() else {
            return Err(crate::Error::InvalidJsonSchemaReference {
                reference: reference.to_owned(),
                schema_uri: resolved_schema_uri,
            });
        };
        return Ok(Some(current_schema_from_document(
            &document_schema,
            schema_view.clone(),
            document_schema.semantic_schema.clone(),
            Cow::Owned(document_schema.definitions.clone()),
            strict,
            parent_dynamic_scope,
        )));
    };

    let reference_with_fragment = format!("#{fragment}");
    if is_plain_name_anchor_reference(&reference_with_fragment) {
        if let Some(mut referable) =
            resolve_from_schema_map(&document_schema.anchors, &reference_with_fragment).await
        {
            return referable
                .resolve_in_scope(
                    Cow::Owned(document_schema.schema_base_uri().clone()),
                    Cow::Owned(document_schema.definitions.clone()),
                    strict,
                    schema_store,
                    Some(parent_dynamic_scope),
                )
                .await
                .map(|result| result.map(CurrentSchema::into_owned));
        }
        return Err(crate::Error::InvalidJsonSchemaReference {
            reference: reference.to_owned(),
            schema_uri: resolved_schema_uri,
        });
    }

    if is_json_pointer(&reference_with_fragment) {
        return resolve_pointer_current_schema(
            schema_store,
            &resolved_schema_uri,
            &reference_with_fragment,
            document_schema.dialect(),
            &document_schema.definitions,
            strict,
            parent_dynamic_scope,
        )
        .await
        .and_then(|resolved| {
            resolved.ok_or(crate::Error::InvalidJsonPointer {
                pointer: reference_with_fragment,
                schema_uri: resolved_schema_uri,
            })
        })
        .map(Some);
    }

    Err(crate::Error::UnsupportedReference {
        reference: reference.to_owned(),
        schema_uri: resolved_schema_uri,
    })
}

fn current_schema_from_document<'a>(
    document_schema: &crate::DocumentSchema,
    schema_view: Arc<SchemaView>,
    semantic_schema: Option<Arc<super::SemanticSchema>>,
    definitions: Cow<'a, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    parent_dynamic_scope: &[SchemaUri],
) -> CurrentSchema<'a> {
    let schema_base_uri = document_schema.schema_base_uri().clone();
    CurrentSchema {
        schema_view,
        semantic_schema,
        schema_uri: Cow::Owned(document_schema.schema_uri.clone()),
        schema_document_uri: Cow::Owned(document_schema.schema_document_uri().clone()),
        definitions,
        strict,
        dynamic_scope: document_schema.dynamic_scope(parent_dynamic_scope),
        schema_base_uri: Cow::Owned(schema_base_uri),
    }
}

async fn resolve_initial_dynamic_anchor_target(
    reference: &str,
    current_base_uri: &SchemaUri,
    schema_store: &crate::SchemaStore,
) -> Result<Option<(Option<SchemaUri>, String)>, crate::Error> {
    let Some((base_schema_uri, dynamic_anchor_ref)) =
        parse_dynamic_anchor_reference(reference, current_base_uri)
    else {
        return Ok(None);
    };
    let lookup_uri = base_schema_uri.as_ref().unwrap_or(current_base_uri);
    let Some(document_schema) = schema_store.try_get_document_schema(lookup_uri).await? else {
        return Ok(None);
    };
    let has_matching_dynamic_anchor = {
        let anchors = document_schema.dynamic_anchors.read().await;
        anchors.contains_key(&dynamic_anchor_ref)
    };
    Ok(has_matching_dynamic_anchor.then_some((base_schema_uri, dynamic_anchor_ref)))
}

/// Extends a parent dynamic scope with `resource_uri` as the new innermost entry.
fn extend_dynamic_scope(parent: &[SchemaUri], resource_uri: &SchemaUri) -> Vec<SchemaUri> {
    if parent.first() == Some(resource_uri) {
        return parent.to_vec();
    }
    let mut scope = Vec::with_capacity(parent.len() + 1);
    scope.push(resource_uri.clone());
    scope.extend_from_slice(parent);
    scope
}

fn declared_dialect(value: &tombi_json::ValueNode) -> Option<crate::JsonSchemaDialect> {
    value
        .as_object()
        .and_then(|object| object.get("$schema"))
        .and_then(tombi_json::ValueNode::as_str)
        .and_then(|dialect_uri| crate::JsonSchemaDialect::try_from(dialect_uri).ok())
}

async fn resolve_pointer_current_schema(
    schema_store: &crate::SchemaStore,
    schema_base_uri: &SchemaUri,
    pointer: &str,
    dialect: Option<crate::JsonSchemaDialect>,
    definitions: &SchemaDefinitions,
    strict: Option<BoolDefaultTrue>,
    parent_dynamic_scope: &[SchemaUri],
) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
    let mut schema_uri = schema_base_uri.clone();
    let mut pointer = pointer.to_owned();
    let mut dialect = dialect;
    let mut definitions = definitions.clone();

    let mut current_dynamic_scope = parent_dynamic_scope.to_vec();
    loop {
        let Some(schema_value) = schema_store.fetch_schema_value(&schema_uri).await? else {
            return Ok(None);
        };
        dialect = dialect
            .or(schema_store.schema_resource_dialect(&schema_uri).await)
            .or_else(|| declared_dialect(&schema_value));

        if let Some((resource_chain, relative_pointer)) =
            resource_boundary_for_pointer(&schema_value, &schema_uri, &pointer)
        {
            let schema_resource_uri = resource_chain.last().cloned().unwrap();
            let mut dynamic_scope_at_target = current_dynamic_scope.clone();
            for resource_uri in &resource_chain {
                dynamic_scope_at_target =
                    extend_dynamic_scope(&dynamic_scope_at_target, resource_uri);
            }
            let Some(resource_document) = schema_store
                .try_get_document_schema(&schema_resource_uri)
                .await?
            else {
                return Err(crate::Error::InvalidJsonPointer {
                    pointer: pointer.clone(),
                    schema_uri,
                });
            };
            if relative_pointer == "#" {
                let Some(schema_view) = resource_document.schema_view.as_ref() else {
                    return Err(crate::Error::InvalidJsonPointer {
                        pointer,
                        schema_uri: schema_resource_uri,
                    });
                };
                return Ok(Some(current_schema_from_document(
                    &resource_document,
                    schema_view.clone(),
                    resource_document.semantic_schema.clone(),
                    Cow::Owned(resource_document.definitions.clone()),
                    strict,
                    &dynamic_scope_at_target,
                )));
            }
            schema_uri = schema_resource_uri;
            pointer = relative_pointer;
            dialect = resource_document.dialect();
            definitions = resource_document.definitions.clone();
            current_dynamic_scope = dynamic_scope_at_target;
            continue;
        }

        let schema_document_uri = schema_store.schema_document_uri_for(&schema_uri).await;
        let Some(schema_view) = resolve_json_pointer(&schema_value, &pointer, None, dialect)?
        else {
            return Err(crate::Error::InvalidJsonPointer {
                pointer,
                schema_uri,
            });
        };
        // Prefer the resource's recorded base URI when this URI is already indexed as a
        // schema resource. Re-joining a relative `$id` against that canonical URI would
        // double-append the relative path (e.g. `…/folder/` + `folder/` → `…/folder/folder/`).
        // Fall back to resolving `$id` only for retrieval URIs that differ from `$id`.
        let schema_base_uri = if let Some(document_schema) =
            schema_store.try_get_document_schema(&schema_uri).await?
        {
            document_schema.schema_base_uri().clone()
        } else {
            schema_value
                .as_object()
                .and_then(|object| object.get("$id"))
                .and_then(tombi_json::ValueNode::as_str)
                .and_then(|id| super::resolve_schema_resource_uri(&schema_uri, id))
                .unwrap_or_else(|| schema_uri.clone())
        };
        let mut instance_uri = schema_document_uri.clone();
        if let Some(fragment) = pointer.strip_prefix('#') {
            instance_uri.set_fragment(Some(fragment));
        }
        return Ok(Some(CurrentSchema {
            schema_view: Arc::new(schema_view),
            semantic_schema: resolve_json_pointer_node(&schema_value, &pointer)
                .and_then(|value| super::SemanticSchema::from_value_node(value, dialect))
                .map(Arc::new),
            schema_uri: Cow::Owned(instance_uri),
            schema_document_uri: Cow::Owned(schema_document_uri),
            definitions: Cow::Owned(definitions),
            strict,
            dynamic_scope: extend_dynamic_scope(&current_dynamic_scope, &schema_base_uri),
            schema_base_uri: Cow::Owned(schema_base_uri),
        }));
    }
}

fn resource_boundary_for_pointer(
    schema_node: &tombi_json::ValueNode,
    schema_base_uri: &SchemaUri,
    pointer: &str,
) -> Option<(Vec<SchemaUri>, String)> {
    if !pointer.starts_with('#') {
        return None;
    }
    let path = &pointer[1..];
    if path.is_empty() {
        return None;
    }

    let decoded_path = percent_decode(path);
    let segments: Vec<&str> = decoded_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    let mut current = schema_node;
    let mut current_base = schema_base_uri.clone();
    if let Some(schema_resource_uri) = current
        .as_object()
        .and_then(|object| object.get("$id"))
        .and_then(tombi_json::ValueNode::as_str)
        .and_then(|id| super::resolve_schema_resource_uri(&current_base, id))
    {
        current_base = schema_resource_uri;
    }
    let mut resource_chain = Vec::new();
    let mut resource_start = 0usize;

    for (index, segment) in segments.iter().enumerate() {
        let decoded_segment = segment.replace("~1", "/").replace("~0", "~");
        match current {
            tombi_json::ValueNode::Object(object) => {
                current = object.get(&decoded_segment)?;
            }
            tombi_json::ValueNode::Array(array) => {
                let index = decoded_segment.parse::<usize>().ok()?;
                current = array.get(index)?;
            }
            _ => return None,
        }
        if let Some(schema_resource_uri) = current
            .as_object()
            .and_then(|object| object.get("$id"))
            .and_then(tombi_json::ValueNode::as_str)
            .and_then(|id| super::resolve_schema_resource_uri(&current_base, id))
        {
            current_base = schema_resource_uri.clone();
            resource_chain.push(schema_resource_uri);
            resource_start = index + 1;
        }
    }

    let schema_resource_uri = resource_chain.last()?;
    if schema_resource_uri == schema_base_uri {
        return None;
    }
    let relative_pointer = if resource_start >= segments.len() {
        "#".to_string()
    } else {
        format!("#/{}", segments[resource_start..].join("/"))
    };
    Some((resource_chain, relative_pointer))
}

fn parse_dynamic_anchor_reference(
    reference: &str,
    current_base_uri: &SchemaUri,
) -> Option<(Option<SchemaUri>, String)> {
    if let Some(fragment) = reference.strip_prefix('#') {
        if !is_plain_name_fragment(&fragment) {
            return None;
        }
        return Some((None, format!("#{fragment}")));
    }

    let mut schema_uri = if let Ok(joined) = current_base_uri.join(reference) {
        SchemaUri::from(joined)
    } else {
        SchemaUri::from_str(reference).ok()?
    };
    let fragment = schema_uri.fragment()?.to_string();
    if !is_plain_name_fragment(&fragment) {
        return None;
    }
    schema_uri.set_fragment(None);
    Some((Some(schema_uri), format!("#{fragment}")))
}

fn is_plain_name_anchor_reference(reference: &str) -> bool {
    if let Some(fragment) = reference.strip_prefix('#') {
        is_plain_name_fragment(fragment)
    } else {
        false
    }
}

#[inline]
fn is_plain_name_fragment(fragment: &str) -> bool {
    !fragment.is_empty() && !fragment.contains('/')
}

pub async fn resolve_and_collect_schemas(
    schemas: &super::ReferableSchemaViews,
    schema_base_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    schema_visits: &crate::SchemaVisits,
    accessors: &[crate::Accessor],
) -> Option<Vec<CurrentSchema<'static>>> {
    resolve_and_collect_schemas_in_scope(
        schemas,
        schema_base_uri,
        definitions,
        strict,
        schema_store,
        schema_visits,
        accessors,
        None,
    )
    .await
}

pub async fn resolve_and_collect_schemas_in_scope(
    schemas: &super::ReferableSchemaViews,
    schema_base_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    schema_visits: &crate::SchemaVisits,
    accessors: &[crate::Accessor],
    parent_dynamic_scope: Option<&[SchemaUri]>,
) -> Option<Vec<CurrentSchema<'static>>> {
    let (collected, errors) = resolve_and_collect_schemas_with_errors_in_scope(
        schemas,
        schema_base_uri,
        definitions,
        strict,
        schema_store,
        schema_visits,
        accessors,
        parent_dynamic_scope,
    )
    .await?;

    for err in errors {
        log::warn!("{err}");
    }

    Some(collected)
}

/// Two-path schema collection: tries a read lock first for already-resolved schemas,
/// resolves refs on cloned entries, and writes back only newly-resolved entries.
///
/// Returns the successfully resolved schemas together with any resolution errors.
/// Returns `None` when schema traversal is re-entrant (cycle guard) or when
/// an initial read lock cannot be acquired due to concurrent mutation.
pub async fn resolve_and_collect_schemas_with_errors(
    schemas: &super::ReferableSchemaViews,
    schema_base_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    schema_visits: &crate::SchemaVisits,
    accessors: &[crate::Accessor],
) -> Option<(Vec<CurrentSchema<'static>>, Vec<crate::Error>)> {
    resolve_and_collect_schemas_with_errors_in_scope(
        schemas,
        schema_base_uri,
        definitions,
        strict,
        schema_store,
        schema_visits,
        accessors,
        None,
    )
    .await
}

pub async fn resolve_and_collect_schemas_with_errors_in_scope(
    schemas: &super::ReferableSchemaViews,
    schema_base_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    schema_visits: &crate::SchemaVisits,
    accessors: &[crate::Accessor],
    parent_dynamic_scope: Option<&[SchemaUri]>,
) -> Option<(Vec<CurrentSchema<'static>>, Vec<crate::Error>)> {
    let Some(_cycle_guard) = schema_visits.get_cycle_guard(schemas) else {
        log::debug!(
            "detected composite schema cycle while collecting schemas: schema_base_uri={schema_base_uri} accessors={accessors} reason=reentrant_schema_traversal",
            schema_base_uri = schema_base_uri.as_ref(),
            accessors = crate::Accessors::from(accessors.to_vec())
        );
        return None;
    };

    let mut schema_entries = Vec::new();
    let resolved_schemas = {
        let Ok(schema_guard) = schemas.try_read() else {
            // try_read() failed -- a write lock is held.
            log::debug!(
                "failed to acquire read lock for composite schema collection: schema_base_uri={schema_base_uri} accessors={accessors} reason=write_lock_held",
                schema_base_uri = schema_base_uri.as_ref(),
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
                            schema_base_uri: resolved_schema_uri,
                            value,
                            semantic_schema,
                        } => Some((
                            resolved_schema_uri.clone(),
                            value.clone(),
                            semantic_schema.clone(),
                        )),
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
        let mut errors = Vec::new();
        let default_schema_base_uri = schema_base_uri.as_ref().clone();
        let default_schema_document_uri = schema_store
            .schema_document_uri_for(&default_schema_base_uri)
            .await;
        let default_definitions = definitions.clone().into_owned();

        for (resolved_schema_uri, schema_view, semantic_schema) in resolved_schemas {
            let (
                current_schema_uri,
                current_schema_base_uri,
                current_schema_document_uri,
                current_definitions,
                current_dynamic_scope,
            ) = if let Some(resolved_schema_uri) = resolved_schema_uri {
                match schema_store
                    .try_get_document_schema(&resolved_schema_uri)
                    .await
                {
                    Ok(Some(document_schema)) => (
                        document_schema.schema_uri.clone(),
                        document_schema.schema_base_uri().clone(),
                        document_schema.schema_document_uri().clone(),
                        document_schema.definitions.clone(),
                        document_schema.dynamic_scope(parent_dynamic_scope.unwrap_or(&[])),
                    ),
                    Ok(None) => (
                        resolved_schema_uri.clone(),
                        default_schema_base_uri.clone(),
                        default_schema_document_uri.clone(),
                        default_definitions.clone(),
                        extend_dynamic_scope(
                            parent_dynamic_scope.unwrap_or(&[]),
                            &default_schema_base_uri,
                        ),
                    ),
                    Err(err) => {
                        errors.push(err);
                        continue;
                    }
                }
            } else {
                (
                    default_schema_document_uri.clone(),
                    default_schema_base_uri.clone(),
                    default_schema_document_uri.clone(),
                    default_definitions.clone(),
                    extend_dynamic_scope(
                        parent_dynamic_scope.unwrap_or(&[]),
                        &default_schema_base_uri,
                    ),
                )
            };

            collected.push(CurrentSchema {
                schema_view,
                semantic_schema,
                schema_uri: Cow::Owned(current_schema_uri),
                schema_document_uri: Cow::Owned(current_schema_document_uri),
                definitions: Cow::Owned(current_definitions),
                strict,
                dynamic_scope: current_dynamic_scope,
                schema_base_uri: Cow::Owned(current_schema_base_uri),
            });
        }

        return Some((collected, errors));
    }

    // Slow path: unresolved refs exist. Resolve on cloned entries and cache back.
    let mut collected = Vec::with_capacity(schema_entries.len());
    let mut errors = Vec::new();
    let mut resolved_indices = Vec::new();
    for (index, referable_schema) in schema_entries.iter_mut().enumerate() {
        let was_ref = referable_schema.is_ref();
        match referable_schema
            .resolve_in_scope(
                schema_base_uri.clone(),
                definitions.clone(),
                strict,
                schema_store,
                parent_dynamic_scope,
            )
            .await
        {
            Ok(Some(current_schema)) => collected.push(current_schema.into_owned()),
            Ok(None) => {}
            Err(err) => {
                errors.push(err);
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
                "failed to acquire write lock for composite schema resolution: schema_base_uri={schema_base_uri} accessors={accessors} reason=lock_contention",
                schema_base_uri = schema_base_uri.as_ref(),
                accessors = crate::Accessors::from(accessors.to_vec())
            );
            return Some((collected, errors));
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

    Some((collected, errors))
}

/// Resolve a schema item without holding its write lock across await points.
///
/// 1. Clone under read lock.
/// 2. If already resolved, build `CurrentSchema` directly.
/// 3. If unresolved, resolve on the cloned item.
/// 4. Write back only the resolved cache state.
pub async fn resolve_schema_item(
    item: &super::SchemaItem,
    schema_base_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
    resolve_schema_item_in_scope(
        item,
        schema_base_uri,
        definitions,
        strict,
        schema_store,
        None,
    )
    .await
}

pub async fn resolve_schema_item_in_scope(
    item: &super::SchemaItem,
    schema_base_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    parent_dynamic_scope: Option<&[SchemaUri]>,
) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
    let mut item_schema = {
        let item_schema = item.read().await;
        if item_schema.is_resolved() {
            return item_schema
                .to_current_schema(
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
                )
                .await;
        }
        item_schema.clone()
    };

    let resolved = item_schema
        .resolve_in_scope(
            schema_base_uri,
            definitions,
            strict,
            schema_store,
            parent_dynamic_scope,
        )
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

/// Resolve a JSON pointer to a SchemaView.
///
/// This function resolves a JSON pointer to a SchemaView.
/// It is used to resolve pointers like `#/properties/foo` within the same schema.
/// More correctly, it should use `#/definitions/foo` to use definitions,
/// but this function is provided for exceptional cases of some JSON Schema implementations.
///
pub fn resolve_json_pointer(
    schema_node: &tombi_json::ValueNode,
    pointer: &str,
    string_formats: Option<&[StringFormat]>,
    dialect: Option<crate::JsonSchemaDialect>,
) -> Result<Option<SchemaView>, crate::Error> {
    let Some(current) = resolve_json_pointer_node(schema_node, pointer) else {
        return Ok(None);
    };

    match current {
        tombi_json::ValueNode::Object(_) => {
            Ok(super::SemanticSchema::from_value_node(current, dialect)
                .and_then(|schema| schema.completion_projection(string_formats)))
        }
        tombi_json::ValueNode::Bool(bool_node) => {
            Ok(Some(bool_schema_view(bool_node.value, bool_node.range)))
        }
        _ => Ok(None),
    }
}

fn resolve_json_pointer_node<'a>(
    schema_node: &'a tombi_json::ValueNode,
    pointer: &str,
) -> Option<&'a tombi_json::ValueNode> {
    if !pointer.starts_with('#') {
        return None;
    }

    let path = &pointer[1..]; // Remove the leading '#'
    if path.is_empty() {
        return Some(schema_node);
    }

    // RFC 6901: Percent-decode the path before splitting on '/'
    let decoded_path = percent_decode(path);
    let segments: Vec<&str> = decoded_path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current = schema_node;

    for segment in segments {
        let decoded_segment = segment.replace("~1", "/").replace("~0", "~");

        match current {
            tombi_json::ValueNode::Object(obj) => {
                current = obj.get(&decoded_segment)?;
            }
            tombi_json::ValueNode::Array(arr) => {
                let index = decoded_segment.parse::<usize>().ok()?;
                current = arr.get(index)?;
            }
            _ => {
                return None;
            }
        }
    }
    Some(current)
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
    use std::{borrow::Cow, path::PathBuf, str::FromStr};

    use crate::{
        Referable, SchemaAccessor, SchemaStore, SchemaView,
        schema::referable_schema::{
            parse_dynamic_anchor_reference, resolve_external_reference, resolve_json_pointer,
        },
    };

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{prefix}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn ref_assertion_siblings_follow_dialect() {
        let schema = tombi_json::ValueNode::from_str(
            r##"{"$ref":"#/$defs/value","type":"string","minLength":3}"##,
        )
        .unwrap();
        let object = schema.as_object().unwrap();

        let draft_7 = Referable::new(
            object,
            None,
            Some(crate::JsonSchemaDialect::Draft07),
            None,
            None,
        );
        std::assert_matches!(
            draft_7,
            Some(Referable::Ref {
                semantic_schema: None,
                ..
            })
        );

        let draft_2020_12 = Referable::new(
            object,
            None,
            Some(crate::JsonSchemaDialect::Draft2020_12),
            None,
            None,
        );
        std::assert_matches!(
            draft_2020_12,
            Some(Referable::Ref {
                semantic_schema: Some(_),
                ..
            })
        );
    }

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
        let result = resolve_json_pointer(
            &value_node,
            "#/foo/bar%2Fbaz",
            None,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            // The schema should be resolved correctly
            std::assert_matches!(schema, SchemaView::String(_));
        }

        // Test case 2: Multiple percent-encoded characters
        let json = r#"{
            "test": {
                "path%2Fwith%20spaces": "value"
            }
        }"#;
        let value_node = ValueNode::from_str(json).unwrap();

        let result = resolve_json_pointer(
            &value_node,
            "#/test/path%2Fwith%20spaces",
            None,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            std::assert_matches!(schema, SchemaView::String(_));
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
        let result = resolve_json_pointer(
            &value_node,
            "#/foo/bar%2",
            None,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let result = resolve_json_pointer(
            &value_node,
            "#/foo/baz%2G",
            None,
            Some(crate::JsonSchemaDialect::Draft07),
        );
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
        let result = resolve_json_pointer(
            &value_node,
            "#/foo/bar~1baz",
            None,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            std::assert_matches!(schema, SchemaView::String(_));
        }

        let result = resolve_json_pointer(
            &value_node,
            "#/foo/qux~0tilde",
            None,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(result.is_ok());
        if let Ok(Some(schema)) = result {
            std::assert_matches!(schema, SchemaView::String(_));
        }
    }

    #[tokio::test]
    async fn test_value_type_ref_does_not_panic() {
        let referable = Referable::Ref {
            reference: "#/definitions/foo".to_string(),
            kind: super::ReferenceKind::Ref,
            semantic_schema: None,
            title: None,
            description: None,
            default: None,
            examples: None,
            deprecation: None,
        };

        let value_type = referable.value_type().await;
        std::assert_matches!(
            value_type,
            crate::ValueType::AnyOf(types) if types.is_empty()
        );
    }

    #[tokio::test]
    async fn test_dynamic_ref_resolves_to_dynamic_anchor_in_scope() {
        let schema_json = r##"{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$dynamicAnchor": "rootDyn",
            "type": "string",
            "$defs": {
                "useDynamic": {
                    "$dynamicRef": "#rootDyn"
                }
            }
        }"##;

        let schema_path = std::env::temp_dir().join(format!(
            "tombi_dynamic_ref_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&schema_path, schema_json).unwrap();

        let schema_uri = tombi_uri::SchemaUri::from_file_path(&schema_path).unwrap();
        let schema_store = SchemaStore::new();
        let document_schema = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();
        let definitions = document_schema.definitions.clone();
        let mut referable = {
            let defs = definitions.read().await;
            defs.get("#/$defs/useDynamic").cloned().unwrap()
        };
        std::assert_matches!(
            referable,
            Referable::Ref {
                kind: super::ReferenceKind::DynamicRef,
                ..
            }
        );

        let resolved = referable
            .resolve(
                Cow::Owned(schema_uri),
                Cow::Owned(definitions),
                None,
                &schema_store,
            )
            .await
            .unwrap();

        std::assert_matches!(
            resolved.map(|s| s.schema_view),
            Some(schema) if matches!(&*schema, SchemaView::String(_))
        );
        let _ = std::fs::remove_file(schema_path);
    }

    #[tokio::test]
    async fn test_recursive_ref_resolves_to_recursive_anchor_in_scope() {
        let schema_json = r##"{
            "$schema": "https://json-schema.org/draft/2019-09/schema",
            "$recursiveAnchor": true,
            "type": "string",
            "$defs": {
                "useRecursive": {
                    "$recursiveRef": "#"
                }
            }
        }"##;

        let schema_path = std::env::temp_dir().join(format!(
            "tombi_recursive_ref_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&schema_path, schema_json).unwrap();

        let schema_uri = tombi_uri::SchemaUri::from_file_path(&schema_path).unwrap();
        let schema_store = SchemaStore::new();
        let document_schema = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();
        let definitions = document_schema.definitions.clone();
        let mut referable = {
            let defs = definitions.read().await;
            defs.get("#/$defs/useRecursive").cloned().unwrap()
        };
        std::assert_matches!(
            referable,
            Referable::Ref {
                kind: super::ReferenceKind::RecursiveRef,
                ..
            }
        );

        let resolved = referable
            .resolve(
                Cow::Owned(schema_uri),
                Cow::Owned(definitions),
                None,
                &schema_store,
            )
            .await
            .unwrap();

        std::assert_matches!(
            resolved.map(|s| s.schema_view),
            Some(schema) if matches!(&*schema, SchemaView::String(_))
        );
        let _ = std::fs::remove_file(schema_path);
    }

    #[tokio::test]
    async fn test_relative_ref_with_external_fragment_resolves() {
        let temp_dir = std::env::temp_dir().join(format!(
            "tombi_referable_schema_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let defs_path = temp_dir.join("defs-source.json");
        let main_path = temp_dir.join("main.json");

        std::fs::write(
            &defs_path,
            r#"{
                "$defs": {
                    "name": { "type": "string" }
                }
            }"#,
        )
        .unwrap();

        std::fs::write(
            &main_path,
            r#"{
                "$defs": {
                    "useExternal": {
                        "$ref": "./defs.json#/$defs/name"
                    }
                }
            }"#,
        )
        .unwrap();

        let renamed_defs_path = main_path.parent().unwrap().join("defs.json");
        std::fs::copy(&defs_path, &renamed_defs_path).unwrap();

        let schema_uri = tombi_uri::SchemaUri::from_file_path(&main_path).unwrap();
        let schema_store = SchemaStore::new();
        let document_schema = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();
        let definitions = document_schema.definitions.clone();
        let mut referable = {
            let defs = definitions.read().await;
            defs.get("#/$defs/useExternal").cloned().unwrap()
        };

        let resolved = referable
            .resolve(
                Cow::Owned(schema_uri),
                Cow::Owned(definitions),
                None,
                &schema_store,
            )
            .await
            .unwrap();

        std::assert_matches!(
            resolved.map(|s| s.schema_view),
            Some(schema) if matches!(&*schema, SchemaView::String(_))
        );

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn external_ref_sibling_keeps_source_and_target_contexts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "tombi_ref_sibling_context_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let main_path = temp_dir.join("main.json");
        let target_path = temp_dir.join("target.json");
        std::fs::write(
            &main_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "local": { "type": "string" },
                    "combined": {
                        "$ref": "./target.json#/$defs/base",
                        "properties": {
                            "local": { "$ref": "#/$defs/local" }
                        }
                    }
                }
            }"##,
        )
        .unwrap();
        std::fs::write(
            &target_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "base": {
                        "type": "object",
                        "properties": { "remote": { "type": "integer" } }
                    }
                }
            }"##,
        )
        .unwrap();

        let source_uri = tombi_uri::SchemaUri::from_file_path(&main_path).unwrap();
        let target_uri = tombi_uri::SchemaUri::from_file_path(&target_path).unwrap();
        let schema_store = SchemaStore::new();
        let document_schema = schema_store
            .try_get_document_schema(&source_uri)
            .await
            .unwrap()
            .unwrap();
        let definitions = document_schema.definitions.clone();
        let mut referable = {
            let defs = definitions.read().await;
            defs.get("#/$defs/combined").cloned().unwrap()
        };

        let resolved = referable
            .resolve(
                Cow::Owned(source_uri.clone()),
                Cow::Owned(definitions),
                None,
                &schema_store,
            )
            .await
            .unwrap()
            .unwrap();
        let SchemaView::AllOf(all_of) = resolved.schema_view.as_ref() else {
            panic!("external $ref with structural siblings must resolve as allOf");
        };
        let branches = all_of.schemas.read().await;

        std::assert_matches!(
            branches.as_slice(),
            [
                Referable::Resolved {
                    schema_base_uri: Some(local_uri),
                    ..
                },
                Referable::Resolved {
                    schema_base_uri: Some(remote_uri),
                    ..
                }
            ] if local_uri == &source_uri && remote_uri == &target_uri
        );
        drop(branches);
        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[tokio::test]
    async fn external_anchor_reference_keeps_parent_dynamic_scope() {
        let temp_dir = unique_temp_dir("tombi_dynamic_scope_anchor");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let outer_path = temp_dir.join("outer.json");
        let inner_path = temp_dir.join("inner.json");
        std::fs::write(
            &outer_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$dynamicAnchor": "value",
                "type": "boolean"
            }"##,
        )
        .unwrap();
        std::fs::write(
            &inner_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "fallback": {
                        "$dynamicAnchor": "value",
                        "type": "string"
                    },
                    "useDynamic": {
                        "$anchor": "useDynamic",
                        "$dynamicRef": "#value"
                    }
                }
            }"##,
        )
        .unwrap();

        let outer_uri = tombi_uri::SchemaUri::from_file_path(&outer_path).unwrap();
        let inner_uri = tombi_uri::SchemaUri::from_file_path(&inner_path).unwrap();
        let schema_store = SchemaStore::new();
        schema_store
            .try_get_document_schema(&outer_uri)
            .await
            .unwrap();
        schema_store
            .try_get_document_schema(&inner_uri)
            .await
            .unwrap();

        let resolved = resolve_external_reference(
            &format!("{inner_uri}#useDynamic"),
            &outer_uri,
            None,
            &schema_store,
            std::slice::from_ref(&outer_uri),
        )
        .await
        .unwrap()
        .unwrap();

        std::assert_matches!(resolved.schema_view.as_ref(), SchemaView::Boolean(_));
        assert!(resolved.dynamic_scope.contains(&inner_uri));
        assert!(resolved.dynamic_scope.contains(&outer_uri));

        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[tokio::test]
    async fn external_pointer_resolution_keeps_parent_dynamic_scope() {
        let temp_dir = unique_temp_dir("tombi_dynamic_scope_pointer");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let outer_path = temp_dir.join("outer.json");
        let inner_path = temp_dir.join("inner.json");
        std::fs::write(
            &outer_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$dynamicAnchor": "value",
                "type": "boolean"
            }"##,
        )
        .unwrap();
        std::fs::write(
            &inner_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "node": {
                        "$id": "shoko://dynamic/node",
                        "type": "object",
                        "properties": {
                            "strict": {
                                "$dynamicRef": "#value"
                            },
                            "values": {
                                "type": "array",
                                "items": {
                                    "$dynamicRef": "#value"
                                }
                            }
                        },
                        "$defs": {
                            "fallback": {
                                "$dynamicAnchor": "value",
                                "type": "string"
                            }
                        }
                    }
                }
            }"##,
        )
        .unwrap();

        let outer_uri = tombi_uri::SchemaUri::from_file_path(&outer_path).unwrap();
        let inner_uri = tombi_uri::SchemaUri::from_file_path(&inner_path).unwrap();
        let schema_store = SchemaStore::new();
        schema_store
            .try_get_document_schema(&outer_uri)
            .await
            .unwrap();
        schema_store
            .try_get_document_schema(&inner_uri)
            .await
            .unwrap();

        let current_schema = resolve_external_reference(
            &format!("{inner_uri}#/$defs/node"),
            &outer_uri,
            None,
            &schema_store,
            std::slice::from_ref(&outer_uri),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(current_schema.dynamic_scope.len(), 2);
        assert_eq!(
            current_schema.dynamic_scope[0],
            tombi_uri::SchemaUri::from_str("shoko://dynamic/node").unwrap()
        );
        assert_eq!(current_schema.dynamic_scope[1], outer_uri);

        let SchemaView::Table(table_schema) = current_schema.schema_view.as_ref() else {
            panic!("external pointer should resolve to a table schema");
        };

        let strict_schema = table_schema
            .resolve_property_schema(
                &SchemaAccessor::Key("strict".to_string()),
                current_schema.schema_base_uri.clone(),
                current_schema.definitions.clone(),
                current_schema.strict,
                &schema_store,
                Some(&current_schema.dynamic_scope),
            )
            .await
            .unwrap()
            .unwrap();
        std::assert_matches!(strict_schema.schema_view.as_ref(), SchemaView::Boolean(_));

        let values_schema = table_schema
            .resolve_property_schema(
                &SchemaAccessor::Key("values".to_string()),
                current_schema.schema_base_uri.clone(),
                current_schema.definitions.clone(),
                current_schema.strict,
                &schema_store,
                Some(&current_schema.dynamic_scope),
            )
            .await
            .unwrap()
            .unwrap();
        let SchemaView::Array(array_schema) = values_schema.schema_view.as_ref() else {
            panic!("values should resolve to an array schema");
        };
        let items = array_schema.items.as_ref().expect("array items");
        let item_schema = crate::resolve_schema_item_in_scope(
            items,
            values_schema.schema_base_uri.clone(),
            values_schema.definitions.clone(),
            values_schema.strict,
            &schema_store,
            Some(&values_schema.dynamic_scope),
        )
        .await
        .unwrap()
        .unwrap();
        std::assert_matches!(item_schema.schema_view.as_ref(), SchemaView::Boolean(_));

        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[tokio::test]
    async fn dynamic_ref_resolution_is_not_cached_across_parent_scopes() {
        let temp_dir = unique_temp_dir("tombi_dynamic_scope_cache");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let outer_boolean_path = temp_dir.join("outer-boolean.json");
        let outer_string_path = temp_dir.join("outer-string.json");
        let inner_path = temp_dir.join("inner.json");
        std::fs::write(
            &outer_boolean_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$dynamicAnchor": "value",
                "type": "boolean"
            }"##,
        )
        .unwrap();
        std::fs::write(
            &outer_string_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$dynamicAnchor": "value",
                "type": "string"
            }"##,
        )
        .unwrap();
        std::fs::write(
            &inner_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "fallback": {
                        "$dynamicAnchor": "value"
                    },
                    "useDynamic": {
                        "$anchor": "useDynamic",
                        "$dynamicRef": "#value"
                    }
                }
            }"##,
        )
        .unwrap();

        let outer_boolean_uri = tombi_uri::SchemaUri::from_file_path(&outer_boolean_path).unwrap();
        let outer_string_uri = tombi_uri::SchemaUri::from_file_path(&outer_string_path).unwrap();
        let inner_uri = tombi_uri::SchemaUri::from_file_path(&inner_path).unwrap();
        let schema_store = SchemaStore::new();
        schema_store
            .try_get_document_schema(&outer_boolean_uri)
            .await
            .unwrap();
        schema_store
            .try_get_document_schema(&outer_string_uri)
            .await
            .unwrap();
        schema_store
            .try_get_document_schema(&inner_uri)
            .await
            .unwrap();

        let first = resolve_external_reference(
            &format!("{inner_uri}#useDynamic"),
            &outer_boolean_uri,
            None,
            &schema_store,
            std::slice::from_ref(&outer_boolean_uri),
        )
        .await
        .unwrap()
        .unwrap();
        std::assert_matches!(first.schema_view.as_ref(), SchemaView::Boolean(_));

        let second = resolve_external_reference(
            &format!("{inner_uri}#useDynamic"),
            &outer_string_uri,
            None,
            &schema_store,
            std::slice::from_ref(&outer_string_uri),
        )
        .await
        .unwrap()
        .unwrap();
        std::assert_matches!(second.schema_view.as_ref(), SchemaView::String(_));

        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[tokio::test]
    async fn external_pointer_keeps_intermediate_resource_scope() {
        let temp_dir = unique_temp_dir("tombi_dynamic_scope_intermediate");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let outer_path = temp_dir.join("outer.json");
        let inner_path = temp_dir.join("inner.json");
        std::fs::write(
            &outer_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object"
            }"##,
        )
        .unwrap();
        std::fs::write(
            &inner_path,
            r##"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "A": {
                        "$id": "https://example.com/A",
                        "$dynamicAnchor": "value",
                        "type": "boolean",
                        "$defs": {
                            "B": {
                                "$id": "https://example.com/B",
                                "$defs": {
                                    "value": {
                                        "$dynamicAnchor": "value"
                                    }
                                },
                                "type": "object",
                                "properties": {
                                    "strict": {
                                        "$dynamicRef": "#value"
                                    }
                                }
                            }
                        }
                    }
                }
            }"##,
        )
        .unwrap();

        let outer_uri = tombi_uri::SchemaUri::from_file_path(&outer_path).unwrap();
        let inner_uri = tombi_uri::SchemaUri::from_file_path(&inner_path).unwrap();
        let schema_store = SchemaStore::new();
        schema_store
            .try_get_document_schema(&outer_uri)
            .await
            .unwrap();
        schema_store
            .try_get_document_schema(&inner_uri)
            .await
            .unwrap();

        let current_schema = resolve_external_reference(
            &format!("{inner_uri}#/$defs/A/$defs/B"),
            &outer_uri,
            None,
            &schema_store,
            std::slice::from_ref(&outer_uri),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(
            current_schema.dynamic_scope,
            vec![
                tombi_uri::SchemaUri::from_str("https://example.com/B").unwrap(),
                tombi_uri::SchemaUri::from_str("https://example.com/A").unwrap(),
                outer_uri.clone(),
            ]
        );

        let SchemaView::Table(table_schema) = current_schema.schema_view.as_ref() else {
            panic!("external pointer should resolve to a table schema");
        };
        let strict_schema = table_schema
            .resolve_property_schema(
                &SchemaAccessor::Key("strict".to_string()),
                current_schema.schema_base_uri.clone(),
                current_schema.definitions.clone(),
                current_schema.strict,
                &schema_store,
                Some(&current_schema.dynamic_scope),
            )
            .await
            .unwrap()
            .unwrap();
        std::assert_matches!(strict_schema.schema_view.as_ref(), SchemaView::Boolean(_));

        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_parse_dynamic_anchor_reference() {
        let base = tombi_uri::SchemaUri::from_str("https://example.com/base/schema.json").unwrap();

        let local = parse_dynamic_anchor_reference("#rootDyn", &base);
        assert_eq!(local, Some((None, "#rootDyn".to_string())));

        let remote =
            parse_dynamic_anchor_reference("https://example.com/schema.json#rootDyn", &base);
        std::assert_matches!(
            remote,
            Some((Some(_), anchor)) if anchor == "#rootDyn"
        );

        let relative = parse_dynamic_anchor_reference("../other.json#rootDyn", &base);
        std::assert_matches!(
            relative,
            Some((Some(uri), anchor))
                if uri.as_str() == "https://example.com/other.json" && anchor == "#rootDyn"
        );

        assert!(parse_dynamic_anchor_reference("#/defs/x", &base).is_none());
    }
}
