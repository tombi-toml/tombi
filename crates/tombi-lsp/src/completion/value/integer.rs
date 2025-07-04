use tombi_extension::CompletionKind;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, IntegerSchema, SchemaUrl};

use crate::completion::{
    CompletionContent, CompletionEdit, CompletionHint, FindCompletionContents,
};

impl FindCompletionContents for IntegerSchema {
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

            if let Some(const_value) = &self.const_value {
                let label = const_value.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_const_value(
                    CompletionKind::Integer,
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
                    let label = item.to_string();
                    let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                    completion_items.push(CompletionContent::new_enumerate_value(
                        CompletionKind::Integer,
                        label,
                        self.title.clone(),
                        self.description.clone(),
                        edit,
                        schema_url,
                        self.deprecated,
                    ));
                }
            }

            if let Some(default) = &self.default {
                let label = default.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_default_value(
                    CompletionKind::Integer,
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_url,
                    self.deprecated,
                ));
            }

            if completion_items.is_empty() {
                completion_items.extend(type_hint_integer(position, schema_url, completion_hint));
            }

            completion_items
        }
        .boxed()
    }
}

pub fn type_hint_integer(
    position: tombi_text::Position,
    schema_url: Option<&SchemaUrl>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let label = "42";
    let edit = CompletionEdit::new_selectable_literal(label, position, completion_hint);

    vec![CompletionContent::new_type_hint_value(
        CompletionKind::Integer,
        label,
        "Integer",
        edit,
        schema_url,
    )]
}
