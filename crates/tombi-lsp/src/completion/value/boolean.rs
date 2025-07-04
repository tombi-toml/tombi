use tombi_extension::CompletionKind;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, BooleanSchema, CurrentSchema, SchemaUrl};

use crate::completion::{
    CompletionContent, CompletionEdit, CompletionHint, FindCompletionContents,
};

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
            let schema_url = current_schema.map(|schema| schema.schema_url.as_ref());

            if let Some(const_value) = &self.const_value {
                let label = const_value.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                return vec![CompletionContent::new_const_value(
                    CompletionKind::Boolean,
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_url,
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
                            schema_url,
                            self.deprecated,
                        )
                    })
                    .collect()
            } else {
                type_hint_boolean(position, schema_url, completion_hint)
            }
        }
        .boxed()
    }
}

pub fn type_hint_boolean(
    position: tombi_text::Position,
    schema_url: Option<&SchemaUrl>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    [true, false]
        .into_iter()
        .map(|value| {
            CompletionContent::new_type_hint_boolean(
                value,
                CompletionEdit::new_literal(&value.to_string(), position, completion_hint),
                schema_url,
            )
        })
        .collect()
}
