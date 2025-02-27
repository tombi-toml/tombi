use std::borrow::Cow;

use futures::{future::BoxFuture, FutureExt};
use schema_store::{Accessor, CurrentSchema, OneOfSchema, SchemaDefinitions, SchemaUrl, Schemas};

use crate::completion::{
    serde_value_to_completion_item, CompletionCandidate, CompletionContent, CompletionHint,
    CompositeSchemaImpl, FindCompletionContents,
};

impl CompositeSchemaImpl for OneOfSchema {
    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn description(&self) -> Option<String> {
        self.description.clone()
    }

    fn schemas(&self) -> &Schemas {
        &self.schemas
    }
}

pub fn find_one_of_completion_items<'a: 'b, 'b, T>(
    value: &'a T,
    position: text::Position,
    keys: &'a [document_tree::Key],
    accessors: &'a [Accessor],
    schema_url: &'a SchemaUrl,
    one_of_schema: &'a schema_store::OneOfSchema,
    definitions: &'a SchemaDefinitions,
    schema_context: &'a schema_store::SchemaContext<'a>,
    completion_hint: Option<CompletionHint>,
) -> BoxFuture<'b, Vec<CompletionContent>>
where
    T: FindCompletionContents + linter::Validate + Sync + Send,
{
    async move {
        let mut completion_items = Vec::new();

        for referable_schema in one_of_schema.schemas.write().await.iter_mut() {
            if let Ok(CurrentSchema {
                value_schema,
                schema_url,
                definitions,
            }) = referable_schema
                .resolve(
                    Cow::Borrowed(schema_url),
                    Cow::Borrowed(definitions),
                    schema_context.store,
                )
                .await
            {
                let schema_completions = value
                    .find_completion_contents(
                        position,
                        keys,
                        accessors,
                        Some(&schema_url),
                        Some(value_schema),
                        Some(&definitions),
                        schema_context,
                        completion_hint,
                    )
                    .await;

                completion_items.extend(schema_completions);
            }
        }

        for completion_item in completion_items.iter_mut() {
            if completion_item.detail.is_none() {
                completion_item.detail = one_of_schema
                    .detail(
                        schema_url,
                        definitions,
                        schema_context.store,
                        completion_hint,
                    )
                    .await;
            }
            if completion_item.documentation.is_none() {
                completion_item.documentation = one_of_schema
                    .documentation(
                        schema_url,
                        definitions,
                        schema_context.store,
                        completion_hint,
                    )
                    .await;
            }
        }

        if let Some(default) = &one_of_schema.default {
            if let Some(completion_item) =
                serde_value_to_completion_item(default, position, Some(schema_url), completion_hint)
            {
                completion_items.push(completion_item);
            }
        }

        completion_items
    }
    .boxed()
}
