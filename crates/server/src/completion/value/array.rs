use std::borrow::Cow;

use document_tree::ArrayKind;
use futures::{future::BoxFuture, FutureExt};
use schema_store::{
    Accessor, ArraySchema, CurrentSchema, SchemaAccessor, SchemaDefinitions, SchemaUrl, ValueSchema,
};

use super::{
    all_of::find_all_of_completion_items, any_of::find_any_of_completion_items,
    one_of::find_one_of_completion_items, type_hint_value, CompletionHint, FindCompletionContents,
};
use crate::completion::{
    completion_kind::CompletionKind, schema_completion::SchemaCompletion, CompletionContent,
    CompletionEdit,
};

impl FindCompletionContents for document_tree::Array {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: text::Position,
        keys: &'a [document_tree::Key],
        accessors: &'a [Accessor],
        schema_url: Option<&'a SchemaUrl>,
        value_schema: Option<&'a ValueSchema>,
        definitions: Option<&'a SchemaDefinitions>,
        schema_context: &'a schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Vec<CompletionContent>> {
        tracing::trace!("self: {:?}", self);
        tracing::trace!("keys: {:?}", keys);
        tracing::trace!("accessors: {:?}", accessors);
        tracing::trace!("value schema: {:?}", value_schema);
        tracing::trace!("completion hint: {:?}", completion_hint);

        async move {
            if let Some(sub_schema_url_map) = schema_context.sub_schema_url_map {
                if let Some(sub_schema_url) = sub_schema_url_map.get(
                    &accessors
                        .iter()
                        .map(SchemaAccessor::from)
                        .collect::<Vec<_>>(),
                ) {
                    if schema_url != Some(sub_schema_url) {
                        if let Ok(document_schema) = schema_context
                            .store
                            .try_load_document_schema(sub_schema_url)
                            .await
                        {
                            return self
                                .find_completion_contents(
                                    position,
                                    keys,
                                    accessors,
                                    Some(&document_schema.schema_url),
                                    document_schema.value_schema.as_ref(),
                                    Some(&document_schema.definitions),
                                    schema_context,
                                    completion_hint,
                                )
                                .await;
                        }
                    }
                }
            }

            if let (Some(schema_url), Some(value_schema), Some(definitions)) =
                (schema_url, value_schema, definitions)
            {
                match value_schema {
                    ValueSchema::Array(array_schema) => {
                        let mut new_item_index = 0;
                        for (index, value) in self.values().iter().enumerate() {
                            if value.range().end() < position {
                                new_item_index = index + 1;
                            }
                            if value.range().contains(position) || value.range().end() == position {
                                let accessor = Accessor::Index(index);
                                if let Some(items) = &array_schema.items {
                                    if let Ok(CurrentSchema {
                                        schema_url,
                                        value_schema,
                                        definitions,
                                    }) = items
                                        .write()
                                        .await
                                        .resolve(
                                            Cow::Borrowed(schema_url),
                                            Cow::Borrowed(definitions),
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
                                                    .collect::<Vec<_>>(),
                                                Some(&schema_url),
                                                Some(value_schema),
                                                Some(&definitions),
                                                schema_context,
                                                completion_hint,
                                            )
                                            .await;
                                    }
                                }
                            }
                        }
                        if let Some(items) = &array_schema.items {
                            if let Ok(CurrentSchema {
                                value_schema,
                                schema_url,
                                definitions,
                            }) = items
                                .write()
                                .await
                                .resolve(
                                    Cow::Borrowed(schema_url),
                                    Cow::Borrowed(definitions),
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
                                            .collect::<Vec<_>>(),
                                        Some(&schema_url),
                                        Some(value_schema),
                                        Some(&definitions),
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
                            schema_url,
                            one_of_schema,
                            definitions,
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
                            schema_url,
                            any_of_schema,
                            definitions,
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
                            schema_url,
                            all_of_schema,
                            definitions,
                            schema_context,
                            completion_hint,
                        )
                        .await
                    }
                    _ => Vec::with_capacity(0),
                }
            } else {
                for (index, value) in self.values().iter().enumerate() {
                    if value.range().contains(position) || value.range().end() == position {
                        if let document_tree::Value::Table(table) = value {
                            if keys.len() == 1 && table.kind() == document_tree::TableKind::KeyValue
                            {
                                let key = &keys.first().unwrap();
                                return vec![CompletionContent::new_type_hint_key(
                                    key,
                                    schema_context.toml_version,
                                    schema_url,
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
                                schema_url,
                                None,
                                definitions,
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
                    schema_url,
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
        position: text::Position,
        _keys: &'a [document_tree::Key],
        _accessors: &'a [Accessor],
        schema_url: Option<&'a SchemaUrl>,
        _value_schema: Option<&'a ValueSchema>,
        _definitions: Option<&'a SchemaDefinitions>,
        _schema_context: &'a schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Vec<CompletionContent>> {
        async move {
            match completion_hint {
                Some(CompletionHint::InTableHeader) => Vec::with_capacity(0),
                _ => type_hint_array(position, schema_url, completion_hint),
            }
        }
        .boxed()
    }
}

pub fn type_hint_array(
    position: text::Position,
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
