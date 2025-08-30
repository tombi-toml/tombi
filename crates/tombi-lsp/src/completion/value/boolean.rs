use tombi_comment_directive::value::BooleanCommonRules;
use tombi_extension::CompletionKind;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, BooleanSchema, CurrentSchema, SchemaUri};

use crate::completion::{
    comment::get_value_comment_directive_completion_contents, CompletionContent, CompletionEdit,
    CompletionHint, FindCompletionContents,
};

impl FindCompletionContents for tombi_document_tree::Boolean {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        _current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        _completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        async move {
            if let Some(comment_directives) = self.comment_directives() {
                for comment_directive in comment_directives {
                    if let Some(completion_contents) =
                        get_value_comment_directive_completion_contents::<BooleanCommonRules>(
                            comment_directive,
                            position,
                            accessors,
                        )
                        .await
                    {
                        return completion_contents;
                    }
                }
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
        _keys: &'a [tombi_document_tree::Key],
        _accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        async move {
            let schema_uri = current_schema.map(|schema| schema.schema_uri.as_ref());

            if let Some(const_value) = &self.const_value {
                let label = const_value.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                return vec![CompletionContent::new_const_value(
                    CompletionKind::Boolean,
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_uri,
                    self.deprecated,
                )];
            }

            if let Some(enumerate) = &self.enumerate {
                enumerate
                    .iter()
                    .map(|value| {
                        let label = value.to_string();
                        let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                        CompletionContent::new_enumerate_value(
                            CompletionKind::Boolean,
                            value.to_string(),
                            self.title.clone(),
                            self.description.clone(),
                            edit,
                            schema_uri,
                            self.deprecated,
                        )
                    })
                    .collect()
            } else {
                type_hint_boolean(position, schema_uri, completion_hint)
            }
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
