use tombi_extension::{CompletionContentPriority, CompletionKind};
use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, CurrentSchema, SchemaAccessor, SchemaContext, SchemaType, SchemaView,
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

        if keys.is_empty() || (keys.len() == 1 && keys[0].span().contains_inclusive(offset)) {
            completion_items =
                filter_all_of_key_completions(completion_items, &resolved_schemas, schema_context)
                    .await;
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

async fn filter_all_of_key_completions(
    completion_items: Vec<CompletionContent>,
    resolved_schemas: &[CurrentSchema<'_>],
    schema_context: &SchemaContext<'_>,
) -> Vec<CompletionContent> {
    if !completion_items
        .iter()
        .any(|item| item.kind == CompletionKind::Key)
    {
        return completion_items;
    }

    let mut object_schemas = Vec::new();
    for schema in resolved_schemas {
        object_schemas.extend(
            collect_all_of_object_schemas(schema.clone().into_owned(), schema_context).await,
        );
    }
    let mut allowed_items = Vec::with_capacity(completion_items.len());
    'item: for item in completion_items {
        if item.kind == CompletionKind::Key {
            let key_name = item.key_name.as_deref();
            for schema in &object_schemas {
                if !schema_allows_key_completion(&item, key_name, schema, schema_context).await {
                    continue 'item;
                }
            }
        }
        allowed_items.push(item);
    }
    allowed_items
}

fn collect_all_of_object_schemas<'a>(
    schema: CurrentSchema<'static>,
    schema_context: &'a SchemaContext<'a>,
) -> tombi_future::BoxFuture<'a, Vec<CurrentSchema<'static>>> {
    async move {
        let schema = schema
            .for_instance_type(SchemaType::Object, schema_context.string_formats())
            .unwrap_or(schema);
        let (all_of, mut conjuncts) = match schema.schema_view.as_ref() {
            SchemaView::AllOf(all_of) => (all_of, Vec::new()),
            SchemaView::Table(table) => {
                let Some(all_of) = table.all_of.as_deref() else {
                    return vec![schema];
                };
                (all_of, vec![schema.clone()])
            }
            _ => return vec![schema],
        };
        let Some(branches) = tombi_schema_store::resolve_and_collect_schemas_in_scope(
            &all_of.schemas,
            schema.schema_base_uri.clone(),
            schema.definitions.clone(),
            schema.strict,
            schema_context.store,
            &schema_context.schema_visits,
            &[],
            Some(&schema.dynamic_scope),
        )
        .await
        else {
            return conjuncts;
        };
        let Some(_cycle_guard) = schema_context
            .schema_visits
            .get_cycle_guard(&all_of.schemas)
        else {
            return conjuncts;
        };
        for branch in branches {
            conjuncts.extend(collect_all_of_object_schemas(branch, schema_context).await);
        }
        conjuncts
    }
    .boxed()
}

async fn schema_allows_key_completion(
    item: &CompletionContent,
    key_name: Option<&str>,
    schema: &CurrentSchema<'_>,
    schema_context: &SchemaContext<'_>,
) -> bool {
    let SchemaView::Table(table) = schema.schema_view.as_ref() else {
        return true;
    };
    if item.priority == CompletionContentPriority::AdditionalKey {
        if table.additional_properties() != Some(false) {
            return true;
        }
        for pattern in pattern_keys(table).await {
            if !pattern_property_forbids_key(table, &pattern, schema, schema_context).await {
                return true;
            }
        }
        return false;
    }
    let Some(key_name) = key_name else {
        return true;
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
    if let Ok(Some(property)) = &property
        && matches!(property.schema_view.as_ref(), SchemaView::Nothing(_))
    {
        return false;
    }
    let matching_patterns = pattern_keys(table)
        .await
        .into_iter()
        .filter(|pattern| {
            tombi_regex::Regex::new(pattern).is_ok_and(|pattern| pattern.is_match(key_name))
        })
        .collect::<Vec<_>>();
    for pattern in &matching_patterns {
        if pattern_property_forbids_key(table, pattern, schema, schema_context).await {
            return false;
        }
    }
    !matches!(property, Ok(None))
        || !matching_patterns.is_empty()
        || table.additional_properties() != Some(false)
}

async fn pattern_keys(table: &tombi_schema_store::TableSchema) -> Vec<String> {
    match &table.pattern_properties {
        Some(patterns) => patterns.read().await.keys().cloned().collect(),
        None => Vec::new(),
    }
}

async fn pattern_property_forbids_key(
    table: &tombi_schema_store::TableSchema,
    pattern: &str,
    schema: &CurrentSchema<'_>,
    schema_context: &SchemaContext<'_>,
) -> bool {
    matches!(
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
        Ok(Some(pattern_schema))
            if matches!(pattern_schema.schema_view.as_ref(), SchemaView::Nothing(_))
    )
}
