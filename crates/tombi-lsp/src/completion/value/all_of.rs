use itertools::Itertools;
use tombi_extension::{CompletionContentPriority, CompletionKind};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, CurrentSchema, SchemaAccessor, SchemaType, SchemaView, SemanticCompositeKind,
    SemanticSchema, TableSchema,
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

/// Drops key candidates of a table that an `allOf` branch (or the table itself) can never accept.
///
/// Each `allOf` branch is validated on its own, so a key rejected by any one of them cannot
/// validate even if another branch (or the table's own `properties`) offers it. `items` must be
/// the key candidates of the table that `all_of_schema` applies to; `own_table` is that table's
/// schema, whose `additionalProperties` also applies to keys contributed by the branches.
pub async fn retain_keys_allowed_by_all_of(
    items: &mut Vec<CompletionContent>,
    all_of_schema: &tombi_schema_store::AllOfSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    accessors: &[Accessor],
    own_table: Option<&TableSchema>,
) {
    let mut restrictions = KeyRestrictions::default();
    restrictions
        .collect_all_of(all_of_schema, current_schema, schema_context, accessors, 0)
        .await;

    let mut allowed_items = Vec::with_capacity(items.len());
    for item in items.drain(..) {
        if key_is_allowed(
            &item,
            &restrictions,
            own_table,
            current_schema,
            schema_context,
        )
        .await
        {
            allowed_items.push(item);
        }
    }
    *items = allowed_items;
}

async fn key_is_allowed(
    item: &CompletionContent,
    restrictions: &KeyRestrictions,
    own_table: Option<&TableSchema>,
    current_schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> bool {
    if item.kind != CompletionKind::Key {
        return true;
    }
    // `$key` placeholders stand for any key matching `additionalProperties` / `patternProperties`.
    let is_placeholder = item.priority == CompletionContentPriority::AdditionalKey;
    let key_name = item.key_name.as_deref();
    if key_name.is_none() && !is_placeholder {
        return true;
    }
    if key_name.is_some_and(|key_name| restrictions.rejects(key_name)) {
        return false;
    }

    for conjunct in &restrictions.conjuncts {
        if let SchemaView::Table(table) = conjunct.schema_view.as_ref()
            && table_rejects_key(table, key_name, conjunct, schema_context).await
        {
            return false;
        }
    }
    if let Some(table) = own_table
        && table_rejects_key(table, key_name, current_schema, schema_context).await
    {
        return false;
    }
    true
}

/// Whether `table` alone rejects the key, or (`key_name == None`) every key a `$key`
/// placeholder could stand for.
async fn table_rejects_key(
    table: &TableSchema,
    key_name: Option<&str>,
    schema: &CurrentSchema<'_>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> bool {
    async fn is_forbidden(
        resolved: Result<Option<CurrentSchema<'static>>, tombi_schema_store::Error>,
    ) -> bool {
        matches!(
            resolved,
            Ok(Some(schema)) if matches!(schema.schema_view.as_ref(), SchemaView::Nothing(_))
        )
    }

    let patterns = match &table.pattern_properties {
        Some(patterns) => patterns.read().await.keys().cloned().collect_vec(),
        None => Vec::new(),
    };
    let mut forbidden_patterns = Vec::new();
    let mut all_patterns_forbidden = true;
    for pattern in &patterns {
        let forbidden = is_forbidden(
            table
                .resolve_pattern_property_schema(
                    pattern,
                    schema.schema_base_uri.clone(),
                    schema.definitions.clone(),
                    schema.strict,
                    schema_context.store,
                    Some(&schema.dynamic_scope),
                )
                .await,
        )
        .await;
        all_patterns_forbidden &= forbidden;
        if forbidden {
            forbidden_patterns.push(pattern.as_str());
        }
    }

    let additional_forbidden = table.additional_properties() == Some(false)
        || match &table.additional_property_schema {
            Some((_, additional_schema)) => {
                is_forbidden(
                    tombi_schema_store::resolve_schema_item_in_scope(
                        additional_schema,
                        schema.schema_base_uri.clone(),
                        schema.definitions.clone(),
                        schema.strict,
                        schema_context.store,
                        Some(&schema.dynamic_scope),
                    )
                    .await,
                )
                .await
            }
            None => false,
        };

    let Some(key_name) = key_name else {
        // A pattern matching every key and forbidding it rejects the placeholder as well.
        if forbidden_patterns
            .iter()
            .any(|pattern| matches!(*pattern, ".*" | "^.*$" | "^.*" | ".*$" | "^"))
        {
            return true;
        }
        return additional_forbidden && all_patterns_forbidden;
    };

    let property = table
        .resolve_property_schema(
            &SchemaAccessor::Key(key_name.to_owned()),
            schema.schema_base_uri.clone(),
            schema.definitions.clone(),
            schema.strict,
            schema_context.store,
            Some(&schema.dynamic_scope),
        )
        .await;
    if is_forbidden(property.clone()).await {
        return true;
    }

    let mut matches_pattern = false;
    let mut has_uncompilable_pattern = false;
    for pattern in &patterns {
        match tombi_regex::Regex::new(pattern) {
            Ok(regex) if regex.is_match(key_name) => {
                matches_pattern = true;
                if forbidden_patterns.contains(&pattern.as_str()) {
                    return true;
                }
            }
            Ok(_) => {}
            Err(_) => has_uncompilable_pattern = true,
        }
    }

    // A key that nothing declares is only valid if `additionalProperties` allows it. An
    // uncompilable pattern cannot be evaluated, so don't reject on its account.
    matches!(property, Ok(None))
        && !matches_pattern
        && !has_uncompilable_pattern
        && additional_forbidden
}

/// Maximum nesting of `allOf` followed while collecting key restrictions.
const MAX_ALL_OF_DEPTH: usize = 16;

/// Key constraints that every instance of an `allOf` must satisfy.
///
/// Only constraints that always apply are followed (`allOf`, `$ref`, and `then` when the
/// `if` only asserts `type: object`); alternatives such as `anyOf` / `oneOf` are skipped.
#[derive(Default)]
struct KeyRestrictions {
    /// `properties: { key: false }`
    forbidden_keys: tombi_hashmap::HashSet<String>,
    /// `patternProperties: { pattern: false }`
    forbidden_patterns: Vec<String>,
    /// `additionalProperties: false`: (declared properties, declared patterns)
    closed_objects: Vec<(Vec<String>, Vec<String>)>,
    /// Resolved `allOf` branches projected for objects, for constraints that only the
    /// runtime view resolves (`$ref` to a `false` schema).
    conjuncts: Vec<CurrentSchema<'static>>,
}

impl KeyRestrictions {
    fn rejects(&self, key: &str) -> bool {
        fn matches_any(patterns: &[String], key: &str) -> bool {
            patterns.iter().any(|pattern| {
                tombi_regex::Regex::new(pattern).is_ok_and(|pattern| pattern.is_match(key))
            })
        }

        self.forbidden_keys.contains(key)
            || matches_any(&self.forbidden_patterns, key)
            || self.closed_objects.iter().any(|(properties, patterns)| {
                !properties.iter().any(|property| property == key) && !matches_any(patterns, key)
            })
    }

    /// Collects the restrictions of every branch of `all_of_schema`, resolving `$ref`s
    /// (including those of nested `allOf`) in their own scope.
    fn collect_all_of<'a: 'b, 'b>(
        &'b mut self,
        all_of_schema: &'b tombi_schema_store::AllOfSchema,
        current_schema: &'b CurrentSchema<'a>,
        schema_context: &'b tombi_schema_store::SchemaContext<'a>,
        accessors: &'b [Accessor],
        depth: usize,
    ) -> BoxFuture<'b, ()> {
        async move {
            if depth >= MAX_ALL_OF_DEPTH {
                return;
            }
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
                return;
            };

            for resolved_schema in &resolved_schemas {
                if let Some(semantic_schema) = &resolved_schema.semantic_schema {
                    self.collect(semantic_schema);
                }
                self.conjuncts.push(
                    resolved_schema
                        .for_instance_type(SchemaType::Object, schema_context.string_formats())
                        .unwrap_or_else(|| resolved_schema.clone()),
                );

                // Nested `allOf` may point to `$ref` targets that the semantic tree
                // does not embed, so follow the runtime view as well.
                let nested_all_of = match resolved_schema.schema_view.as_ref() {
                    SchemaView::AllOf(all_of_schema) => Some(all_of_schema),
                    schema_view => schema_view.adjacent_applicators().2,
                };
                if let Some(nested_all_of) = nested_all_of {
                    self.collect_all_of(
                        nested_all_of,
                        resolved_schema,
                        schema_context,
                        accessors,
                        depth + 1,
                    )
                    .await;
                }
            }
        }
        .boxed()
    }

    fn collect(&mut self, schema: &SemanticSchema) {
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
                if object.ignores_reference_siblings() {
                    return;
                }
                let constraints = &object.constraints.object;
                let is_false = |schema: &SemanticSchema| matches!(schema, SemanticSchema::Boolean(value) if !value.value);

                for (key, property) in &constraints.properties {
                    if is_false(&property.schema) {
                        self.forbidden_keys.insert(key.clone());
                    }
                }
                for (pattern, property) in &constraints.pattern_properties {
                    if is_false(&property.schema) {
                        self.forbidden_patterns.push(pattern.clone());
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
                        constraints.properties.keys().cloned().collect(),
                        constraints.pattern_properties.keys().cloned().collect(),
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
