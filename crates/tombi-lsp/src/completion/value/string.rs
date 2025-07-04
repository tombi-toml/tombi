use tombi_extension::CompletionKind;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUrl, StringSchema};

use crate::completion::{
    schema_completion::SchemaCompletion, CompletionContent, CompletionEdit, CompletionHint,
    FindCompletionContents,
};

impl FindCompletionContents for tombi_document_tree::String {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        async move {
            if !self.range().contains(position) {
                return Vec::with_capacity(0);
            }

            let current_string_value = self.value();

            SchemaCompletion
                .find_completion_contents(
                    position,
                    keys,
                    accessors,
                    current_schema,
                    schema_context,
                    completion_hint,
                )
                .await
                .into_iter()
                .filter_map(|mut completion_content| {
                    if completion_content.kind != CompletionKind::String {
                        return None;
                    }

                    if !completion_content
                        .label
                        .trim_matches('"')
                        .starts_with(current_string_value)
                    {
                        return None;
                    }

                    completion_content.edit = CompletionEdit::new_string_literal_while_editing(
                        &completion_content.label,
                        self.range(),
                    );

                    Some(completion_content)
                })
                .collect()
        }
        .boxed()
    }
}

impl FindCompletionContents for StringSchema {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        _accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        async move {
            let mut completion_items = vec![];
            let schema_url = current_schema.map(|schema| schema.schema_url.as_ref());

            if let Some(default) = &self.default {
                let label = format!("\"{default}\"");
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_default_value(
                    CompletionKind::String,
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_url,
                    self.deprecated,
                ));
            }

            if let Some(const_value) = &self.const_value {
                let label = format!("\"{const_value}\"");
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_const_value(
                    CompletionKind::String,
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_url,
                    self.deprecated,
                ));
                return completion_items;
            }

            if let Some(enumerate) = &self.enumerate {
                for item in enumerate {
                    let label = format!("\"{item}\"");
                    let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                    completion_items.push(CompletionContent::new_enumerate_value(
                        CompletionKind::String,
                        label,
                        self.title.clone(),
                        self.description.clone(),
                        edit,
                        schema_url,
                        self.deprecated,
                    ));
                }
                return completion_items;
            }

            completion_items.extend(
                type_hint_string(position, schema_url, completion_hint)
                    .into_iter()
                    .filter(|completion_content| {
                        self.default
                            .as_ref()
                            .map(|default| default != &completion_content.label)
                            .unwrap_or(true)
                    }),
            );

            completion_items
        }
        .boxed()
    }
}

pub fn type_hint_string(
    position: tombi_text::Position,
    schema_url: Option<&SchemaUrl>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    [('\"', "BasicString"), ('\'', "LiteralString")]
        .into_iter()
        .map(|(quote, detail)| {
            CompletionContent::new_type_hint_string(
                CompletionKind::String,
                quote,
                detail,
                CompletionEdit::new_string_literal(quote, position, completion_hint),
                schema_url,
            )
        })
        .collect()
}
