use futures::{future::BoxFuture, FutureExt};
use schema_store::{Accessor, CurrentSchema, LocalDateSchema, SchemaUrl};

use crate::completion::{
    completion_kind::CompletionKind, CompletionContent, CompletionEdit, CompletionHint,
    FindCompletionContents,
};

impl FindCompletionContents for LocalDateSchema {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        _accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Vec<CompletionContent>> {
        async move {
            let mut completion_items = vec![];
            let schema_url = current_schema.map(|schema| schema.schema_url.as_ref());

            if let Some(enumerate) = &self.enumerate {
                for item in enumerate {
                    let label = item.to_string();
                    let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                    completion_items.push(CompletionContent::new_enumerate_value(
                        CompletionKind::LocalDate,
                        label,
                        self.title.clone(),
                        self.description.clone(),
                        edit,
                        schema_url,
                    ));
                }
            }

            if let Some(default) = &self.default {
                let label = default.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_default_value(
                    CompletionKind::LocalDate,
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_url,
                ));
            }

            if completion_items.is_empty() {
                completion_items.extend(type_hint_local_date(
                    position,
                    schema_url,
                    completion_hint,
                ));
            }

            completion_items
        }
        .boxed()
    }
}

pub fn type_hint_local_date(
    position: tombi_text::Position,
    schema_url: Option<&SchemaUrl>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let label = chrono::Local::now().format("%Y-%m-%d").to_string();
    let edit = CompletionEdit::new_selectable_literal(&label, position, completion_hint);

    vec![CompletionContent::new_type_hint_value(
        CompletionKind::LocalDate,
        label,
        "LocalDate",
        edit,
        schema_url,
    )]
}
