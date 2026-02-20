use tombi_extension::CompletionContentPriority;
use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, CurrentSchema, SchemaAccessor, ValueSchema,
};

use crate::completion::{
    CompletionCandidate, CompletionContent, CompletionHint, FindCompletionContents,
    tombi_json_value_to_completion_default_item, tombi_json_value_to_completion_example_item,
};

pub fn find_one_of_completion_items<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [Accessor],
    one_of_schema: &'a tombi_schema_store::OneOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    completion_hint: Option<CompletionHint>,
) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>>
where
    T: FindCompletionContents + tombi_validator::Validate + Sync + Send + std::fmt::Debug,
{
    log::trace!("value = {:?}", value);
    log::trace!("position = {:?}", position);
    log::trace!("keys = {:?}", keys);
    log::trace!("accessors = {:?}", accessors);
    log::trace!("one_of_schema = {:?}", one_of_schema);
    log::trace!("completion_hint = {:?}", completion_hint);

    async move {
        let mut completion_items = Vec::new();

        // When we are completing the value of a single key (e.g. license = { file = "..." }),
        // only consider branches that are a table containing that key. Otherwise we would merge
        // completions from all branches (e.g. file path and string variant like "MIT").
        // Require exactly one key so we do not narrow when completing after a dot (e.g. "path."
        // yields keys = ["path", ""]). Skip oneOf-level default/examples when narrowing.
        // Only narrow when at least one branch has the key, so we never return [] by over-narrowing.
        let first_key = (keys.len() == 1 && !keys[0].value.is_empty()).then(|| &keys[0].value);

        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            &one_of_schema.schemas,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
        )
        .await
        else {
            return completion_items;
        };

        let Ok(_cycle_guard) = one_of_schema.schemas.try_write() else {
            return completion_items;
        };

        let mut branch_results: Vec<(bool, Vec<CompletionContent>)> = Vec::new();
        for resolved_schema in &resolved_schemas {
            let branch_has_key = if let Some(ref first_key) = first_key {
                match resolved_schema.value_schema.as_ref() {
                    ValueSchema::Table(table_schema) => table_schema
                        .properties
                        .read()
                        .await
                        .contains_key(&SchemaAccessor::Key(first_key.to_string())),
                    _ => false,
                }
            } else {
                false
            };

            let schema_completions = value
                .find_completion_contents(
                    position,
                    keys,
                    accessors,
                    Some(resolved_schema),
                    schema_context,
                    completion_hint,
                )
                .await;

            branch_results.push((branch_has_key, schema_completions));
        }

        let narrow_branches = branch_results.iter().any(|(has_key, _)| *has_key);
        for (branch_has_key, items) in branch_results {
            if !narrow_branches || branch_has_key {
                completion_items.extend(items);
            }
        }

        drop(_cycle_guard);

        let detail = one_of_schema
            .detail(
                &current_schema.schema_uri,
                &current_schema.definitions,
                schema_context.store,
                completion_hint,
            )
            .await;

        let documentation = one_of_schema
            .documentation(
                &current_schema.schema_uri,
                &current_schema.definitions,
                schema_context.store,
                completion_hint,
            )
            .await;

        for completion_item in completion_items.iter_mut() {
            if completion_item.detail.is_none() {
                completion_item.detail = detail.clone();
            }
            if completion_item.documentation.is_none() {
                completion_item.documentation = documentation.clone();
            }
        }

        if !narrow_branches {
            if let Some(default) = &one_of_schema.default {
                let default_label = default.to_string();
                if let Some(completion_item) = completion_items
                    .iter_mut()
                    .find(|item| item.label == default_label)
                {
                    completion_item.priority = CompletionContentPriority::Default;
                } else if let Some(completion_item) = tombi_json_value_to_completion_default_item(
                    default,
                    position,
                    detail.clone(),
                    documentation.clone(),
                    Some(&current_schema.schema_uri),
                    completion_hint,
                ) {
                    completion_items.push(completion_item);
                }
            }

            if let Some(examples) = &one_of_schema.examples {
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
                        position,
                        detail.clone(),
                        documentation.clone(),
                        Some(&current_schema.schema_uri),
                        completion_hint,
                    ) {
                        completion_items.push(completion_item);
                    }
                }
            }
        }

        completion_items
    }
    .boxed()
}
