use tombi_extension::CompletionContentPriority;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema};

use crate::completion::{
    CompletionCandidate, CompletionContent, CompletionHint, FindCompletionContents,
    tombi_json_value_to_completion_default_item, tombi_json_value_to_completion_example_item,
};

use super::branch_result::collect_branch_completions;

pub fn find_any_of_completion_items<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [Accessor],
    any_of_schema: &'a tombi_schema_store::AnyOfSchema,
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
    log::trace!("any_of_schema = {:?}", any_of_schema);
    log::trace!("completion_hint = {:?}", completion_hint);

    async move {
        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            &any_of_schema.schemas,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
        )
        .await
        else {
            return Vec::new();
        };

        let (mut completion_items, narrow_branches) = collect_branch_completions(
            value,
            position,
            keys,
            accessors,
            &resolved_schemas,
            schema_context,
            completion_hint,
        )
        .await;

        let detail = any_of_schema
            .detail(
                &current_schema.schema_uri,
                &current_schema.definitions,
                schema_context.store,
                completion_hint,
            )
            .await;

        let documentation = any_of_schema
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

        if !narrow_branches && keys.is_empty() {
            if let Some(default) = &any_of_schema.default {
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

            if let Some(examples) = &any_of_schema.examples {
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
