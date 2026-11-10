use tombi_extension::{CompletionContentPriority, CompletionKind};
use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, CurrentSchema, SchemaType, SemanticCompositeKind, SemanticSchema,
};

use crate::completion::{
    CompletionContent, CompletionHint, FindCompletionContents, dedup_composite_completion_contents,
    take_completion_schema_tooltip, tombi_json_value_to_completion_default_item,
    tombi_json_value_to_completion_example_item,
};

pub fn find_all_of_completion_items<'a: 'b, 'b, T>(
    value: &'a T,
    cursor: crate::CursorPosition,
    keys: &'a [tombi_document_tree_syntax::Key<'_>],
    accessors: &'a [Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    completion_hint: Option<CompletionHint>,
) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>>
where
    T: FindCompletionContents + Sync + Send + std::fmt::Debug,
{
    let offset = cursor.offset();
    log::trace!("value = {:?}", value);
    log::trace!("offset = {:?}", offset);
    log::trace!("keys = {:?}", keys);
    log::trace!("accessors = {:?}", accessors);
    log::trace!("all_of_schema = {:?}", all_of_schema);
    log::trace!("completion_hint = {:?}", completion_hint);

    async move {
        let mut completion_items = Vec::new();

        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas_in_scope(
            &all_of_schema.schemas,
            current_schema.schema_base_uri.clone(),
            current_schema.definitions.clone(),
            current_schema.strict,
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
            Some(&current_schema.dynamic_scope),
        )
        .await
        else {
            return Vec::new();
        };

        for resolved_schema in &resolved_schemas {
            let schema_completions = value
                .find_completion_contents(
                    cursor,
                    keys,
                    accessors,
                    Some(resolved_schema),
                    schema_context,
                    completion_hint,
                )
                .await;

            completion_items.extend(schema_completions.into_iter().map(|mut item| {
                let tooltip = take_completion_schema_tooltip(&mut item, resolved_schema);
                (item, tooltip)
            }));
        }

        let mut completion_items = dedup_composite_completion_contents(completion_items);

        // Each `allOf` branch is validated on its own, so a key rejected by any one of
        // them can never validate, even if another branch offers it as a candidate.
        let mut restrictions = KeyRestrictions::default();
        for resolved_schema in &resolved_schemas {
            if let Some(semantic_schema) = &resolved_schema.semantic_schema {
                restrictions.collect(semantic_schema);
            }
        }
        // With non-empty `keys` the items belong to a deeper table, which these
        // restrictions do not describe.
        if keys.is_empty() && !restrictions.is_empty() {
            completion_items.retain(|item| {
                // Placeholders (`$key`) stand for any key, not for a declared property.
                item.kind != CompletionKind::Key
                    || item.priority == CompletionContentPriority::AdditionalKey
                    || !restrictions.rejects(&item.label)
            });
        }

        let detail = crate::completion::composite_detail(
            all_of_schema,
            &current_schema.schema_base_uri,
            &current_schema.definitions,
            current_schema.strict,
            schema_context.store,
            &current_schema.dynamic_scope,
            completion_hint,
        )
        .await;
        let documentation = crate::completion::composite_documentation(
            all_of_schema,
            &current_schema.schema_base_uri,
            &current_schema.definitions,
            current_schema.strict,
            schema_context.store,
            &current_schema.dynamic_scope,
            completion_hint,
        )
        .await;

        if keys.is_empty() {
            for completion_item in completion_items.iter_mut() {
                if completion_item.detail.is_none() {
                    completion_item.detail = detail.clone();
                }
                if completion_item.documentation.is_none() {
                    completion_item.documentation = documentation.clone();
                }
            }
        }

        if keys.is_empty()
            && let Some(default) = &all_of_schema.default
        {
            let default_label = default.to_string();
            if let Some(completion_item) = completion_items
                .iter_mut()
                .find(|item| item.label == default_label)
            {
                completion_item.priority = CompletionContentPriority::Default;
            } else if let Some(completion_item) = tombi_json_value_to_completion_default_item(
                default,
                offset,
                detail.clone(),
                documentation.clone(),
                Some(&current_schema.schema_base_uri),
                completion_hint,
            ) {
                completion_items.push(completion_item);
            }
        }

        if keys.is_empty()
            && let Some(examples) = &all_of_schema.examples
        {
            for example in examples {
                let example_label = example.to_string();
                if completion_items
                    .iter()
                    .any(|item| item.label == example_label)
                {
                    continue;
                }

                if let Some(completion_item) = tombi_json_value_to_completion_example_item(
                    example,
                    offset,
                    detail.clone(),
                    documentation.clone(),
                    Some(&current_schema.schema_base_uri),
                    completion_hint,
                ) {
                    completion_items.push(completion_item);
                }
            }
        }

        completion_items
    }
    .boxed()
}

/// Key constraints that every instance of an `allOf` must satisfy.
///
/// Only constraints that always apply are followed (`allOf`, `$ref`, and `then` when the
/// `if` only asserts `type: object`); alternatives such as `anyOf` / `oneOf` are skipped.
#[derive(Default)]
struct KeyRestrictions<'a> {
    /// `properties: { key: false }`
    forbidden_keys: tombi_hashmap::HashSet<&'a str>,
    /// `patternProperties: { pattern: false }`
    forbidden_patterns: Vec<&'a str>,
    /// `additionalProperties: false`: (declared properties, declared patterns)
    closed_objects: Vec<(Vec<&'a str>, Vec<&'a str>)>,
}

impl<'a> KeyRestrictions<'a> {
    fn is_empty(&self) -> bool {
        self.forbidden_keys.is_empty()
            && self.forbidden_patterns.is_empty()
            && self.closed_objects.is_empty()
    }

    fn rejects(&self, key: &str) -> bool {
        fn matches_any(patterns: &[&str], key: &str) -> bool {
            patterns.iter().any(|pattern| {
                tombi_regex::Regex::new(pattern).is_ok_and(|pattern| pattern.is_match(key))
            })
        }

        self.forbidden_keys.contains(key)
            || matches_any(&self.forbidden_patterns, key)
            || self.closed_objects.iter().any(|(properties, patterns)| {
                !properties.contains(&key) && !matches_any(patterns, key)
            })
    }

    fn collect(&mut self, schema: &'a SemanticSchema) {
        match schema {
            SemanticSchema::Boolean(_) => {}
            SemanticSchema::Composite(composite) => {
                if matches!(
                    composite.kind,
                    SemanticCompositeKind::AllOf | SemanticCompositeKind::Reference
                ) {
                    for schema in &composite.schemas {
                        self.collect(schema);
                    }
                }
            }
            SemanticSchema::Object(object) => {
                let constraints = &object.constraints.object;
                let is_false = |schema: &SemanticSchema| matches!(schema, SemanticSchema::Boolean(value) if !value.value);

                for (key, property) in &constraints.properties {
                    if is_false(&property.schema) {
                        self.forbidden_keys.insert(key.as_str());
                    }
                }
                for (pattern, property) in &constraints.pattern_properties {
                    if is_false(&property.schema) {
                        self.forbidden_patterns.push(pattern.as_str());
                    }
                }
                // A pattern that fails to compile cannot be evaluated, so don't treat the
                // object as closed rather than hide keys it might allow.
                if constraints
                    .additional_properties
                    .as_deref()
                    .is_some_and(is_false)
                    && constraints
                        .pattern_properties
                        .keys()
                        .all(|pattern| tombi_regex::Regex::new(pattern).is_ok())
                {
                    self.closed_objects.push((
                        constraints.properties.keys().map(String::as_str).collect(),
                        constraints
                            .pattern_properties
                            .keys()
                            .map(String::as_str)
                            .collect(),
                    ));
                }

                for schema in &object.applicators.all_of {
                    self.collect(schema);
                }
                if let (Some(SemanticSchema::Object(if_schema)), Some(then_schema)) = (
                    object.applicators.if_schema.as_deref(),
                    object.applicators.then_schema.as_deref(),
                ) && if_schema.only_asserts_type(SchemaType::Object)
                {
                    self.collect(then_schema);
                }
            }
        }
    }
}
