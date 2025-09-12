use ahash::AHashSet;
use itertools::Itertools;
use std::borrow::Cow;
use tombi_document_tree::{ArrayKind, LiteralValueRef};
use tombi_extension::{AddLeadingComma, AddTrailingComma, CompletionKind};
use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, ArraySchema, CurrentSchema, DocumentSchema, SchemaUri, ValueSchema,
};

use super::{
    all_of::find_all_of_completion_items, any_of::find_any_of_completion_items,
    one_of::find_one_of_completion_items, type_hint_value, CompletionHint, FindCompletionContents,
};
use crate::{
    comment_directive::get_array_comment_directive_content_with_schema_uri,
    completion::{
        comment::get_tombi_comment_directive_content_completion_contents,
        schema_completion::SchemaCompletion, CompletionContent, CompletionEdit,
    },
};

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
            if keys.is_empty() {
                if let Some((comment_directive_context, schema_uri)) =
                    get_array_comment_directive_content_with_schema_uri(self, position, accessors)
                {
                    if let Some(completions) =
                        get_tombi_comment_directive_content_completion_contents(
                            comment_directive_context,
                            schema_uri,
                        )
                        .await
                    {
                        return completions;
                    }
                }
            }

            if let Some(Ok(DocumentSchema {
                value_schema: Some(value_schema),
                schema_uri,
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
                            schema_uri: Cow::Owned(schema_uri),
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
                        let mut new_item_start_position = None;
                        for (index, value) in self.values().iter().enumerate() {
                            if value.range().end < position {
                                new_item_index = index + 1;
                                new_item_start_position = Some(value.range().end);
                            }
                            if value.contains(position) {
                                let accessor = Accessor::Index(index);
                                if let Some(items) = &array_schema.items {
                                    if let Ok(Some(current_schema)) = items
                                        .write()
                                        .await
                                        .resolve(
                                            current_schema.schema_uri.clone(),
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
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                let mut completions = SchemaCompletion
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
                                            if new_item_index == 0 {
                                                let add_trailing_comma = if self.len() == 0
                                                    || matches!(
                                                        completion_hint,
                                                        Some(CompletionHint::Comma {
                                                            trailing_comma: Some(_),
                                                            ..
                                                        })
                                                    ) {
                                                    None
                                                } else {
                                                    Some(AddTrailingComma)
                                                };

                                                Some(CompletionHint::InArray {
                                                    add_leading_comma: None,
                                                    add_trailing_comma,
                                                })
                                            } else {
                                                let add_leading_comma = if matches!(
                                                    completion_hint,
                                                    Some(CompletionHint::Comma {
                                                        leading_comma: Some(_),
                                                        ..
                                                    })
                                                ) {
                                                    None
                                                } else if let Some(start_position) =
                                                    new_item_start_position
                                                {
                                                    Some(AddLeadingComma { start_position })
                                                } else {
                                                    None
                                                };

                                                let add_trailing_comma = if matches!(
                                                    completion_hint,
                                                    Some(CompletionHint::Comma {
                                                        trailing_comma: Some(_),
                                                        ..
                                                    })
                                                ) {
                                                    None
                                                } else {
                                                    if new_item_index != self.len() {
                                                        Some(AddTrailingComma)
                                                    } else {
                                                        None
                                                    }
                                                };

                                                Some(CompletionHint::InArray {
                                                    add_leading_comma,
                                                    add_trailing_comma,
                                                })
                                            }
                                        } else {
                                            completion_hint
                                        },
                                    )
                                    .await;

                                if array_schema.unique_items == Some(true) {
                                    let unique_values = self
                                        .values()
                                        .iter()
                                        .filter_map(Option::<LiteralValueRef>::from)
                                        .map(|value| value.to_string())
                                        .collect::<AHashSet<_>>();

                                    completions = completions
                                        .into_iter()
                                        .filter(|completion| {
                                            !(completion.kind.is_literal()
                                                && unique_values.contains(&completion.label))
                                        })
                                        .collect_vec();
                                }

                                return completions;
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
                    if value.contains(position) {
                        // Array of tables
                        if let tombi_document_tree::Value::Table(table) = value {
                            if keys.len() == 1
                                && table.kind() == tombi_document_tree::TableKind::KeyValue
                            {
                                let key = &keys.first().unwrap();
                                return vec![CompletionContent::new_type_hint_key(
                                    &key.value,
                                    key.range(),
                                    None,
                                    Some(CompletionHint::InArray {
                                        add_leading_comma: None,
                                        add_trailing_comma: None,
                                    }),
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
                                    .collect_vec(),
                                None,
                                schema_context,
                                completion_hint,
                            )
                            .await;
                    }
                }
                type_hint_value(None, position, None, completion_hint)
            }
        }
        .boxed()
    }
}

impl FindCompletionContents for ArraySchema {
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
            match completion_hint {
                Some(CompletionHint::InTableHeader) => Vec::with_capacity(0),
                _ => {
                    let schema_uri = current_schema.map(|schema| schema.schema_uri.as_ref());

                    let mut completion_items =
                        type_hint_array(position, schema_uri, completion_hint);

                    if let Some(default) = &self.default {
                        let label = default.to_string();
                        let edit = CompletionEdit::new_literal(&label, position, completion_hint);
                        completion_items.push(CompletionContent::new_default_value(
                            CompletionKind::Integer,
                            label,
                            self.title.clone(),
                            self.description.clone(),
                            edit,
                            schema_uri,
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
    schema_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let edit = CompletionEdit::new_array_literal(position, completion_hint);

    vec![CompletionContent::new_type_hint_value(
        CompletionKind::Array,
        "[]",
        "Array",
        edit,
        schema_uri,
    )]
}
