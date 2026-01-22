use tombi_comment_directive::value::{BooleanCommonFormatRules, BooleanCommonLintRules};
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, BooleanSchema, CurrentSchema, SchemaUri};

use crate::{
    comment_directive::get_key_table_value_comment_directive_content_and_schema_uri,
    completion::{
        CompletionContent, CompletionEdit, CompletionHint, FindCompletionContents,
        comment::get_tombi_comment_directive_content_completion_contents,
    },
};

impl FindCompletionContents for tombi_document_tree::Boolean {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        tracing::trace!("self = {:?}", self);
        tracing::trace!("position = {:?}", position);
        tracing::trace!("keys = {:?}", keys);
        tracing::trace!("accessors = {:?}", accessors);
        tracing::trace!("current_schema = {:?}", current_schema);
        tracing::trace!("completion_hint = {:?}", completion_hint);

        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_key_table_value_comment_directive_content_and_schema_uri::<
                    BooleanCommonFormatRules,
                    BooleanCommonLintRules,
                >(self.comment_directives(), position, accessors)
                && let Some(completions) = get_tombi_comment_directive_content_completion_contents(
                    comment_directive_context,
                    schema_uri,
                )
                .await
            {
                return completions;
            }

            Vec::with_capacity(0)
        }
        .boxed()
    }
}

impl FindCompletionContents for BooleanSchema {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        tracing::trace!("self = {:?}", self);
        tracing::trace!("position = {:?}", position);
        tracing::trace!("keys = {:?}", keys);
        tracing::trace!("accessors = {:?}", accessors);
        tracing::trace!("current_schema = {:?}", current_schema);
        tracing::trace!("completion_hint = {:?}", completion_hint);

        async move {
            let schema_uri = current_schema.map(|schema| schema.schema_uri.as_ref());
            let mut completion_items = Vec::new();

            if let Some(const_value) = &self.const_value {
                let label = const_value.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_const_value(
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_uri,
                    self.deprecated,
                ));

                return completion_items;
            }

            if let Some(r#enum) = &self.r#enum {
                completion_items.extend(r#enum.iter().map(|value| {
                    let label = value.to_string();
                    let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                    CompletionContent::new_enum_value(
                        value.to_string(),
                        self.title.clone(),
                        self.description.clone(),
                        edit,
                        schema_uri,
                        self.deprecated,
                    )
                }));

                return completion_items;
            }

            if let Some(examples) = &self.examples {
                for example in examples {
                    let label = example.to_string();
                    if completion_items.iter().any(|item| item.label == label) {
                        continue;
                    }
                    let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                    completion_items.push(CompletionContent::new_example_value(
                        label,
                        self.title.clone(),
                        self.description.clone(),
                        edit,
                        schema_uri,
                        self.deprecated,
                    ));
                }
            }

            if completion_items.is_empty() {
                completion_items = type_hint_boolean(position, schema_uri, completion_hint);
            }

            completion_items
        }
        .boxed()
    }
}

pub fn type_hint_boolean(
    position: tombi_text::Position,
    schema_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    [true, false]
        .into_iter()
        .map(|value| {
            CompletionContent::new_type_hint_boolean(
                value,
                CompletionEdit::new_literal(&value.to_string(), position, completion_hint),
                schema_uri,
            )
        })
        .collect()
}
