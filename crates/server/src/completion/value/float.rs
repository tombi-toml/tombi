use config::TomlVersion;
use futures::{future::BoxFuture, FutureExt};
use schema_store::{Accessor, FloatSchema, SchemaDefinitions, SchemaStore, SchemaUrl, ValueSchema};

use crate::completion::{
    completion_kind::CompletionKind, CompletionContent, CompletionEdit, CompletionHint,
    FindCompletionContents,
};

impl FindCompletionContents for FloatSchema {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a Vec<Accessor>,
        _value_schema: Option<&'a ValueSchema>,
        _toml_version: TomlVersion,
        position: text::Position,
        _keys: &'a [document_tree::Key],
        schema_url: Option<&'a SchemaUrl>,
        _definitions: Option<&'a SchemaDefinitions>,
        _sub_schema_url_map: Option<&'a schema_store::SubSchemaUrlMap>,
        _schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Vec<CompletionContent>> {
        async move {
            let mut completion_items = vec![];

            if let Some(enumerate) = &self.enumerate {
                for item in enumerate {
                    let label = item.to_string();
                    let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                    completion_items.push(CompletionContent::new_enumerate_value(
                        CompletionKind::Float,
                        label,
                        edit,
                        schema_url,
                    ));
                }
            }

            if let Some(default) = &self.default {
                let label = default.to_string();
                let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                completion_items.push(CompletionContent::new_default_value(
                    CompletionKind::Float,
                    label,
                    edit,
                    schema_url,
                ));
            }

            if completion_items.is_empty() {
                completion_items.extend(type_hint_float(position, schema_url, completion_hint));
            }

            completion_items
        }
        .boxed()
    }
}

pub fn type_hint_float(
    position: text::Position,
    schema_url: Option<&SchemaUrl>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let label = "3.14";
    let edit = CompletionEdit::new_selectable_literal(label, position, completion_hint);
    vec![CompletionContent::new_type_hint_value(
        CompletionKind::Float,
        label,
        "Float",
        edit,
        schema_url,
    )]
}
