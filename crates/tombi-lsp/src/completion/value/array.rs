use itertools::Itertools;
use std::borrow::Cow;
use tombi_document_tree::ArrayKind;
use tombi_extension::CompletionKind;
use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, ArraySchema, CurrentSchema, DocumentSchema, SchemaUrl, ValueSchema,
};

use super::{
    all_of::find_all_of_completion_items, any_of::find_any_of_completion_items,
    one_of::find_one_of_completion_items, type_hint_value, CompletionHint, FindCompletionContents,
};
use crate::completion::{schema_completion::SchemaCompletion, CompletionContent, CompletionEdit};

impl FindCompletionContents for tombi_document_tree::Array {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        tracing::trace!("self = {:?}", self);
        tracing::trace!("keys = {:?}", keys);
        tracing::trace!("accessors = {:?}", accessors);
        tracing::trace!("current_schema = {:?}", current_schema);
        tracing::trace!("completion_hint = {:?}", completion_hint);

        async move {
            if let Some(Ok(DocumentSchema {
                value_schema: Some(value_schema),
                schema_url,
                definitions,
                ..
            })) = schema_context
                .get_subschema(accessors, current_schema)
                .await
            {
                return self
                    .find_completion_contents(
                        position,
                        keys,
                        accessors,
                        Some(&CurrentSchema {
                            value_schema: Cow::Owned(value_schema),
                            schema_url: Cow::Owned(schema_url),
                            definitions: Cow::Owned(definitions),
                        }),
                        schema_context,
                        completion_hint,
                    )
                    .await;
            }

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Array(array_schema) => {
                        let mut new_item_index = 0;
                        for (index, value) in self.values().iter().enumerate() {
                            if value.range().end < position {
                                new_item_index = index + 1;
                            }
                            if value.range().contains(position) {
                                let accessor = Accessor::Index(index);
                                if let Some(items) = &array_schema.items {
                                    if let Ok(Some(current_schema)) = items
                                        .write()
                                        .await
                                        .resolve(
                                            current_schema.schema_url.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        return value
                                            .find_completion_contents(
                                                position,
                                                keys,
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor))
                                                    .collect_vec(),
                                                Some(&current_schema),
                                                schema_context,
                                                completion_hint,
                                            )
                                            .await;
                                    }
                                }
                            }
                        }
                        if let Some(items) = &array_schema.items {
                            if let Ok(Some(current_schema)) = items
                                .write()
                                .await
                                .resolve(
                                    current_schema.schema_url.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                return SchemaCompletion
                                    .find_completion_contents(
                                        position,
                                        keys,
                                        &accessors
                                            .iter()
                                            .cloned()
                                            .chain(std::iter::once(Accessor::Index(new_item_index)))
                                            .collect_vec(),
                                        Some(&current_schema),
                                        schema_context,
                                        if self.kind() == ArrayKind::Array {
                                            Some(CompletionHint::InArray)
                                        } else {
                                            completion_hint
                                        },
                                    )
                                    .await;
                            }
                        }

                        Vec::with_capacity(0)
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        find_one_of_completion_items(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        find_any_of_completion_items(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        find_all_of_completion_items(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                    }
                    _ => Vec::with_capacity(0),
                }
            } else {
                for (index, value) in self.values().iter().enumerate() {
                    if value.range().contains(position) {
                        if let tombi_document_tree::Value::Table(table) = value {
                            if keys.len() == 1
                                && table.kind() == tombi_document_tree::TableKind::KeyValue
                            {
                                let key = &keys.first().unwrap();
                                return vec![CompletionContent::new_type_hint_key(
                                    &key.to_raw_text(schema_context.toml_version),
                                    key.range(),
                                    None,
                                    Some(CompletionHint::InArray),
                                )];
                            }
                        }

                        let accessor = Accessor::Index(index);
                        return value
                            .find_completion_contents(
                                position,
                                keys,
                                &accessors
                                    .iter()
                                    .cloned()
                                    .chain(std::iter::once(accessor))
                                    .collect::<Vec<_>>(),
                                None,
                                schema_context,
                                completion_hint,
                            )
                            .await;
                    }
                }
                type_hint_value(
                    None,
                    position,
                    schema_context.toml_version,
                    None,
                    completion_hint,
                )
            }
        }
        .boxed()
    }
}

impl FindCompletionContents for ArraySchema {
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
            match completion_hint {
                Some(CompletionHint::InTableHeader) => Vec::with_capacity(0),
                _ => {
                    let schema_url = current_schema.map(|schema| schema.schema_url.as_ref());

                    let mut completion_items =
                        type_hint_array(position, schema_url, completion_hint);

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

                    completion_items
                }
            }
        }
        .boxed()
    }
}

pub fn type_hint_array(
    position: tombi_text::Position,
    schema_url: Option<&SchemaUrl>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let edit = CompletionEdit::new_array_literal(position, completion_hint);

    vec![CompletionContent::new_type_hint_value(
        CompletionKind::Array,
        "[]",
        "Array",
        edit,
        schema_url,
    )]
}
