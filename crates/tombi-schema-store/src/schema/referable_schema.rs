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
        schema_uri: Option<SchemaUri>,
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
    pub schema_uri: Cow<'a, SchemaUri>,
    pub definitions: Cow<'a, SchemaDefinitions>,
    /// strict setting on root-schema/sub-schema level.
    pub strict: Option<BoolDefaultTrue>,
}

impl<'a> CurrentSchema<'a> {
    pub fn into_owned(self) -> CurrentSchema<'static> {
        CurrentSchema {
            schema_view: self.schema_view,
            semantic_schema: self.semantic_schema,
            schema_uri: Cow::Owned(self.schema_uri.into_owned()),
            definitions: Cow::Owned(self.definitions.into_owned()),
            strict: self.strict,
        }
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
            definitions: Cow::Owned(self.definitions.as_ref().clone()),
            strict: self.strict,
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
        Some(self.with_projected_view(
            semantic_schema,
            semantic_schema.schema_view_for_type(instance_type, string_formats),
        ))
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
            Some(Referable::Ref {
                reference: reference.to_string(),
                kind,
                semantic_schema: Some(Arc::new(super::SemanticSchema::from_object_node(
                    object, dialect,
                ))),
                title: object
                    .get("title")
                    .and_then(|title| title.as_str().map(|s| s.to_string())),
                description: object
                    .get("description")
                    .and_then(|description| description.as_str().map(|s| s.to_string())),
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
                schema_uri: None,
                value: Arc::new(schema_view),
                semantic_schema,
            })
        };

        if let Some(referable) = referable.as_ref() {
            super::update_named_anchors(
                object,
                referable,
                dialect,
                anchor_collector,
                dynamic_anchor_collector,
            );
        }

        referable
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self, Referable::Resolved { .. })
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
        schema_uri: Cow<'a, SchemaUri>,
        definitions: Cow<'a, SchemaDefinitions>,
        strict: Option<BoolDefaultTrue>,
        schema_store: &'a crate::SchemaStore,
    ) -> tombi_future::BoxFuture<'b, Result<Option<CurrentSchema<'a>>, crate::Error>> {
        let dynamic_scope = vec![schema_uri.as_ref().clone()];
        self.resolve_with_dynamic_scope(
            schema_uri,
            definitions,
            strict,
            schema_store,
            dynamic_scope,
        )
    }

    fn resolve_with_dynamic_scope<'a: 'b, 'b>(
        &'a mut self,
        schema_uri: Cow<'a, SchemaUri>,
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
                        ReferenceKind::DynamicRef => parse_dynamic_anchor_reference(reference),
                        ReferenceKind::RecursiveRef => parse_recursive_anchor_reference(reference),
                        ReferenceKind::Ref => None,
                    };
                    if let Some((base_schema_uri, dynamic_anchor_ref)) = dynamic_target {
                        let mut scope_for_dynamic_ref = dynamic_scope.clone();
                        if let Some(base_schema_uri) = base_schema_uri {
                            scope_for_dynamic_ref.insert(0, base_schema_uri);
                        }
                        if let Some((mut referable_schema, owner_schema_uri, owner_definitions)) =
                            resolve_dynamic_anchor_from_scope(
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
                            *self = referable_schema;
                            return self
                                .resolve_with_dynamic_scope(
                                    Cow::Owned(owner_schema_uri),
                                    Cow::Owned(owner_definitions),
                                    strict,
                                    schema_store,
                                    scope_for_dynamic_ref,
                                )
                                .await;
                        }
                    }

                    let definition_schema =
                        { resolve_from_schema_map(&definitions, reference).await };
                    let anchor_schema = if definition_schema.is_none() {
                        resolve_anchor_reference(reference, &schema_uri, schema_store).await?
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

                        *self = referable_schema;
                    } else if is_json_pointer(reference) {
                        let pointer = reference;

                        // Exceptional handling for schemas that do not use `#/definitions/*`.
                        // Therefore, schema_value is not cached in memory, but read from file cache.
                        // Execution speed decreases, but memory usage can be reduced.
                        if let Some(schema_value) =
                            schema_store.fetch_schema_value(&schema_uri).await?
                        {
                            let dialect = schema_value
                                .as_object()
                                .and_then(|object| object.get("$schema"))
                                .and_then(|value| value.as_str())
                                .and_then(|dialect_uri| {
                                    crate::JsonSchemaDialect::try_from(dialect_uri).ok()
                                });
                            if let Some(mut resolved_schema) =
                                resolve_json_pointer(&schema_value, pointer, None, dialect)?
                            {
                                if title.is_some() || description.is_some() {
                                    resolved_schema.set_title(title.to_owned());
                                    resolved_schema.set_description(description.to_owned());
                                }
                                if let Some(default) = default {
                                    resolved_schema.set_default(Some(default.clone()));
                                }
                                if let Some(examples) = examples {
                                    resolved_schema.set_examples(Some(examples.clone()));
                                }
                                if let Some(deprecation) = deprecation {
                                    resolved_schema.set_deprecation(deprecation.clone());
                                }

                                return Ok(Some(CurrentSchema {
                                    schema_view: Arc::new(resolved_schema),
                                    semantic_schema: resolve_json_pointer_node(
                                        &schema_value,
                                        pointer,
                                    )
                                    .and_then(|value| {
                                        super::SemanticSchema::from_value_node(value, dialect)
                                    })
                                    .map(Arc::new)
                                    .map(|target| {
                                        combine_ref_semantics(
                                            ref_semantic_schema.clone(),
                                            Some(target),
                                        )
                                        .expect("resolved reference has semantic schema")
                                    }),
                                    schema_uri: Cow::Owned(schema_uri.as_ref().clone()),
                                    definitions: Cow::Owned(definitions.clone().into_owned()),
                                    strict,
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
                    } else if let Some(resolved_reference) = resolve_external_reference(
                        reference,
                        schema_uri.as_ref(),
                        strict,
                        schema_store,
                    )
                    .await?
                    {
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

                        *self = Referable::Resolved {
                            schema_uri: Some(resolved_reference.schema_uri.as_ref().clone()),
                            value: resolved_value,
                            semantic_schema: combine_ref_semantics(
                                ref_semantic_schema.clone(),
                                resolved_reference.semantic_schema.clone(),
                            ),
                        };
                        let mut dynamic_scope = dynamic_scope.clone();
                        dynamic_scope.insert(0, resolved_reference.schema_uri.as_ref().clone());

                        return self
                            .resolve_with_dynamic_scope(
                                Cow::Owned(resolved_reference.schema_uri.into_owned()),
                                Cow::Owned(resolved_reference.definitions.into_owned()),
                                resolved_reference.strict,
                                schema_store,
                                dynamic_scope,
                            )
                            .await;
                    } else {
                        return Err(crate::Error::UnsupportedReference {
                            reference: reference.to_owned(),
                            schema_uri: schema_uri.as_ref().to_owned(),
                        });
                    }

                    self.resolve_with_dynamic_scope(
                        schema_uri,
                        definitions,
                        strict,
                        schema_store,
                        dynamic_scope,
                    )
                    .await
                }
                Referable::Resolved {
                    schema_uri: reference_url,
                    value: schema_view,
                    semantic_schema,
                } => {
                    let (schema_uri, definitions) = {
                        match reference_url {
                            Some(reference_url) => {
                                if let Some(document_schema) =
                                    schema_store.try_get_document_schema(reference_url).await?
                                {
                                    (
                                        Cow::Owned(document_base_uri(&document_schema)),
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
                        schema_view: schema_view.clone(),
                        semantic_schema: semantic_schema.clone(),
                        schema_uri,
                        definitions,
                        strict,
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
        strict: Option<BoolDefaultTrue>,
        schema_store: &crate::SchemaStore,
    ) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
        match self {
            Referable::Ref { .. } => Ok(None),
            Referable::Resolved {
                schema_uri: reference_url,
                value: schema_view,
                semantic_schema,
            } => {
                let (schema_uri, definitions) = match reference_url {
                    Some(reference_url) => {
                        if let Some(document_schema) =
                            schema_store.try_get_document_schema(reference_url).await?
                        {
                            (
                                Cow::Owned(document_base_uri(&document_schema)),
                                Cow::Owned(document_schema.definitions.clone()),
                            )
                        } else {
                            (schema_uri, definitions)
                        }
                    }
                    None => (schema_uri, definitions),
                };

                Ok(Some(CurrentSchema {
                    schema_view: schema_view.clone(),
                    semantic_schema: semantic_schema.clone(),
                    schema_uri: Cow::Owned(schema_uri.into_owned()),
                    definitions: Cow::Owned(definitions.into_owned()),
                    strict,
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
            super::SemanticCompositeKind::AllOf,
            vec![local.as_ref().clone(), target.as_ref().clone()],
            local.range(),
        ))),
        (Some(schema), None) | (None, Some(schema)) => Some(schema),
        (None, None) => None,
    }
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
    schema_uri: &SchemaUri,
    schema_store: &crate::SchemaStore,
) -> Result<Option<Referable<SchemaView>>, crate::Error> {
    if !is_plain_name_anchor_reference(reference) {
        return Ok(None);
    }
    let Some(document_schema) = schema_store.try_get_document_schema(schema_uri).await? else {
        return Ok(None);
    };
    Ok(resolve_from_schema_map(&document_schema.anchors, reference).await)
}

async fn resolve_dynamic_anchor_from_scope(
    reference: &str,
    dynamic_scope: &[SchemaUri],
    schema_store: &crate::SchemaStore,
) -> Result<Option<(Referable<SchemaView>, SchemaUri, SchemaDefinitions)>, crate::Error> {
    for scope_schema_uri in dynamic_scope {
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
                document_base_uri(&document_schema),
                document_schema.definitions.clone(),
            )));
        }
    }

    Ok(None)
}

async fn resolve_external_reference(
    reference: &str,
    base_schema_uri: &SchemaUri,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
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
        return Ok(Some(CurrentSchema {
            schema_view: schema_view.clone(),
            semantic_schema: document_schema.semantic_schema.clone(),
            schema_uri: Cow::Owned(document_base_uri(&document_schema)),
            definitions: Cow::Owned(document_schema.definitions.clone()),
            strict,
        }));
    };

    let reference_with_fragment = format!("#{fragment}");
    if is_plain_name_anchor_reference(&reference_with_fragment) {
        if let Some(mut referable) =
            resolve_from_schema_map(&document_schema.anchors, &reference_with_fragment).await
        {
            return referable
                .resolve(
                    Cow::Owned(document_base_uri(&document_schema)),
                    Cow::Owned(document_schema.definitions.clone()),
                    strict,
                    schema_store,
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
        if let Some(schema_value) = schema_store
            .fetch_schema_value(&resolved_schema_uri)
            .await?
        {
            let dialect = schema_value
                .as_object()
                .and_then(|object| object.get("$schema"))
                .and_then(|value| value.as_str())
                .and_then(|dialect_uri| crate::JsonSchemaDialect::try_from(dialect_uri).ok());
            if let Some(schema_view) =
                resolve_json_pointer(&schema_value, &reference_with_fragment, None, dialect)?
            {
                return Ok(Some(CurrentSchema {
                    schema_view: Arc::new(schema_view),
                    semantic_schema: resolve_json_pointer_node(
                        &schema_value,
                        &reference_with_fragment,
                    )
                    .and_then(|value| super::SemanticSchema::from_value_node(value, dialect))
                    .map(Arc::new),
                    schema_uri: Cow::Owned(document_base_uri(&document_schema)),
                    definitions: Cow::Owned(document_schema.definitions.clone()),
                    strict,
                }));
            }
        }
        return Err(crate::Error::InvalidJsonPointer {
            pointer: reference_with_fragment,
            schema_uri: resolved_schema_uri,
        });
    }

    Err(crate::Error::UnsupportedReference {
        reference: reference.to_owned(),
        schema_uri: resolved_schema_uri,
    })
}

fn document_base_uri(document_schema: &crate::DocumentSchema) -> SchemaUri {
    document_schema.base_uri().clone()
}

fn parse_dynamic_anchor_reference(reference: &str) -> Option<(Option<SchemaUri>, String)> {
    if let Some(fragment) = reference.strip_prefix('#') {
        if !is_plain_name_fragment(fragment) {
            return None;
        }
        return Some((None, format!("#{fragment}")));
    }

    let (base_uri, fragment) = reference.split_once('#')?;
    if !is_plain_name_fragment(fragment) {
        return None;
    }

    let base_schema_uri = SchemaUri::from_str(base_uri).ok()?;
    Some((Some(base_schema_uri), format!("#{fragment}")))
}

fn parse_recursive_anchor_reference(reference: &str) -> Option<(Option<SchemaUri>, String)> {
    if reference == "#" {
        Some((None, "#".to_string()))
    } else {
        None
    }
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
    schema_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    schema_visits: &crate::SchemaVisits,
    accessors: &[crate::Accessor],
) -> Option<Vec<CurrentSchema<'static>>> {
    let (collected, errors) = resolve_and_collect_schemas_with_errors(
        schemas,
        schema_uri,
        definitions,
        strict,
        schema_store,
        schema_visits,
        accessors,
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
    schema_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
    schema_visits: &crate::SchemaVisits,
    accessors: &[crate::Accessor],
) -> Option<(Vec<CurrentSchema<'static>>, Vec<crate::Error>)> {
    let Some(_cycle_guard) = schema_visits.get_cycle_guard(schemas) else {
        log::debug!(
            "detected composite schema cycle while collecting schemas: schema_uri={schema_uri} accessors={accessors} reason=reentrant_schema_traversal",
            schema_uri = schema_uri.as_ref(),
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
                schema_uri = schema_uri.as_ref(),
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
        let default_schema_uri = schema_uri.as_ref().clone();
        let default_definitions = definitions.clone().into_owned();

        for (resolved_schema_uri, schema_view, semantic_schema) in resolved_schemas {
            let (current_schema_uri, current_definitions) =
                if let Some(resolved_schema_uri) = resolved_schema_uri {
                    match schema_store
                        .try_get_document_schema(&resolved_schema_uri)
                        .await
                    {
                        Ok(Some(document_schema)) => (
                            document_base_uri(&document_schema),
                            document_schema.definitions.clone(),
                        ),
                        Ok(None) => (default_schema_uri.clone(), default_definitions.clone()),
                        Err(err) => {
                            errors.push(err);
                            continue;
                        }
                    }
                } else {
                    (default_schema_uri.clone(), default_definitions.clone())
                };

            collected.push(CurrentSchema {
                schema_view,
                semantic_schema,
                schema_uri: Cow::Owned(current_schema_uri),
                definitions: Cow::Owned(current_definitions),
                strict,
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
            .resolve(
                schema_uri.clone(),
                definitions.clone(),
                strict,
                schema_store,
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
                "failed to acquire write lock for composite schema resolution: schema_uri={schema_uri} accessors={accessors} reason=lock_contention",
                schema_uri = schema_uri.as_ref(),
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
    schema_uri: Cow<'_, SchemaUri>,
    definitions: Cow<'_, SchemaDefinitions>,
    strict: Option<BoolDefaultTrue>,
    schema_store: &crate::SchemaStore,
) -> Result<Option<CurrentSchema<'static>>, crate::Error> {
    let mut item_schema = {
        let item_schema = item.read().await;
        if item_schema.is_resolved() {
            return item_schema
                .to_current_schema(schema_uri, definitions, strict, schema_store)
                .await;
        }
        item_schema.clone()
    };

    let resolved = item_schema
        .resolve(
            schema_uri.clone(),
            definitions.clone(),
            strict,
            schema_store,
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
    use std::{borrow::Cow, str::FromStr};

    use crate::{
        Referable, SchemaStore, SchemaView,
        schema::referable_schema::{parse_dynamic_anchor_reference, resolve_json_pointer},
    };

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
        std::assert_matches!(value_type, crate::ValueType::AnyOf(types) if types.is_empty());
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

    #[test]
    fn test_parse_dynamic_anchor_reference() {
        let local = parse_dynamic_anchor_reference("#rootDyn");
        assert_eq!(local, Some((None, "#rootDyn".to_string())));

        let remote = parse_dynamic_anchor_reference("https://example.com/schema.json#rootDyn");
        std::assert_matches!(remote, Some((Some(_), anchor)) if anchor == "#rootDyn");

        assert!(parse_dynamic_anchor_reference("#/defs/x").is_none());
    }
}
