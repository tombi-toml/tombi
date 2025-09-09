use tombi_comment_directive::value::FloatCommonRules;
use tombi_extension::CompletionKind;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, FloatSchema, SchemaUri};

use crate::{
    comment_directive::get_key_table_value_comment_directive_content_and_schema_uri,
    completion::{
        comment::get_tombi_comment_directive_content_completion_contents, CompletionContent,
        CompletionEdit, CompletionHint, FindCompletionContents,
    },
};

impl FindCompletionContents for tombi_document_tree::Float {
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
                get_key_table_value_comment_directive_content_and_schema_uri::<FloatCommonRules>(
                    self.comment_directives(),
                    position,
                    accessors,
                )
            {
                if let Some(completions) = get_tombi_comment_directive_content_completion_contents(
                    comment_directive_context,
                    schema_uri,
                )
                .await
                {
                    return completions;
                }
            }

            Vec::with_capacity(0)
        }
        .boxed()
    }
}

impl FindCompletionContents for FloatSchema {
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
            let mut completion_items = vec![];
            let schema_uri = current_schema.map(|schema| schema.schema_uri.as_ref());

            if let Some(const_value) = &self.const_value {
                let label = const_value.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_const_value(
                    CompletionKind::Float,
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_uri,
                    self.deprecated,
                ));
                return completion_items;
            }

            if let Some(enumerate) = &self.enumerate {
                for item in enumerate {
                    let label = item.to_string();
                    let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                    completion_items.push(CompletionContent::new_enumerate_value(
                        CompletionKind::Float,
                        label,
                        self.title.clone(),
                        self.description.clone(),
                        edit,
                        schema_uri,
                        self.deprecated,
                    ));
                }
            }

            if let Some(default) = &self.default {
                let label = default.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_default_value(
                    CompletionKind::Float,
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_uri,
                    self.deprecated,
                ));
            }

            if completion_items.is_empty() {
                completion_items.extend(type_hint_float(position, schema_uri, completion_hint));
            }

            completion_items
        }
        .boxed()
    }
}

pub fn type_hint_float(
    position: tombi_text::Position,
    schema_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let label = "3.14";
    let edit = CompletionEdit::new_selectable_literal(label, position, completion_hint);
    vec![CompletionContent::new_type_hint_value(
        CompletionKind::Float,
        label,
        "Float",
        edit,
        schema_uri,
    )]
}
