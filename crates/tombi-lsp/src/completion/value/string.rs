use tombi_comment_directive::value::{StringCommonFormatRules, StringCommonLintRules};
use tombi_extension::CompletionKind;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUri, StringSchema};

use crate::{
    comment_directive::get_key_table_value_comment_directive_content_and_schema_uri,
    completion::{
        CompletionContent, CompletionEdit, CompletionHint, FindCompletionContents,
        comment::get_tombi_comment_directive_content_completion_contents,
        merge_adjacent_schema_completion_items, schema_completion::SchemaCompletion,
    },
};

impl FindCompletionContents for tombi_document_tree_syntax::String<'_> {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        cursor: crate::CursorPosition<'a>,
        keys: &'a [tombi_document_tree_syntax::Key<'_>],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        let offset = cursor.offset();
        log::trace!("self = {:?}", self);
        log::trace!("keys = {:?}", keys);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("current_schema = {:?}", current_schema);
        log::trace!("completion_hint = {:?}", completion_hint);

        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_key_table_value_comment_directive_content_and_schema_uri::<
                    StringCommonFormatRules,
                    StringCommonLintRules,
                >(self.comment_directives(), offset, accessors)
                && let Some(completions) = get_tombi_comment_directive_content_completion_contents(
                    comment_directive_context,
                    schema_uri,
                )
                .await
            {
                return completions;
            }

            if !self.span().contains_inclusive(offset) {
                return Vec::new();
            }

            let current_string_value = self.value();

            if let Some(current_schema) = current_schema {
                SchemaCompletion
                    .find_completion_contents(
                        cursor,
                        keys,
                        accessors,
                        Some(current_schema),
                        schema_context,
                        completion_hint,
                    )
                    .await
                    .into_iter()
                    .filter_map(|mut completion_content| {
                        if !matches!(
                            completion_content.kind,
                            CompletionKind::String | CompletionKind::Enum
                        ) {
                            return None;
                        }

                        if matches!(
                            completion_content.label.as_str(),
                            "\"\"" | "''" | "\"\"\"\"\"\"" | "''''''"
                        ) {
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
                            self.span(),
                        );

                        Some(completion_content)
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
        .boxed()
    }
}

impl FindCompletionContents for StringSchema {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        cursor: crate::CursorPosition<'a>,
        _keys: &'a [tombi_document_tree_syntax::Key<'_>],
        _accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        let offset = cursor.offset();
        async move {
            let mut completion_items = vec![];
            let schema_base_uri = current_schema.map(|schema| schema.schema_base_uri.as_ref());

            if let Some(default) = &self.default {
                let label = format!("\"{default}\"");
                let edit = CompletionEdit::new_literal(&label, offset, completion_hint);
                completion_items.push(CompletionContent::new_default_value(
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_base_uri,
                    self.deprecated(),
                ));
            }

            if let Some(const_value) = &self.const_value {
                let label = format!("\"{const_value}\"");
                let edit = CompletionEdit::new_literal(&label, offset, completion_hint);
                completion_items.push(CompletionContent::new_const_value(
                    label,
                    self.title.clone(),
                    self.description.clone(),
                    edit,
                    schema_base_uri,
                    self.deprecated(),
                ));
                return merge_adjacent_schema_completion_items(
                    cursor,
                    _keys,
                    _accessors,
                    current_schema,
                    _schema_context,
                    completion_hint,
                    completion_items,
                    self.one_of.as_deref(),
                    self.any_of.as_deref(),
                    self.all_of.as_deref(),
                )
                .await;
            }

            if let Some(r#enum) = &self.r#enum {
                for item in r#enum {
                    let label = format!("\"{item}\"");
                    let edit = CompletionEdit::new_literal(&label, offset, completion_hint);
                    completion_items.push(CompletionContent::new_enum_value(
                        label,
                        self.title.clone(),
                        self.description.clone(),
                        edit,
                        schema_base_uri,
                        self.deprecated(),
                    ));
                }
                return merge_adjacent_schema_completion_items(
                    cursor,
                    _keys,
                    _accessors,
                    current_schema,
                    _schema_context,
                    completion_hint,
                    completion_items,
                    self.one_of.as_deref(),
                    self.any_of.as_deref(),
                    self.all_of.as_deref(),
                )
                .await;
            }

            if let Some(examples) = &self.examples {
                for example in examples {
                    let label = format!("\"{example}\"");
                    if completion_items.iter().any(|item| item.label == label) {
                        continue;
                    }
                    let edit = CompletionEdit::new_literal(&label, offset, completion_hint);
                    completion_items.push(CompletionContent::new_example_value(
                        label,
                        self.title.clone(),
                        self.description.clone(),
                        edit,
                        schema_base_uri,
                        self.deprecated(),
                    ));
                }
            }

            completion_items.extend(
                type_hint_string(offset, schema_base_uri, completion_hint)
                    .into_iter()
                    .filter(|completion_content| {
                        self.default
                            .as_ref()
                            .map(|default| default != &completion_content.label)
                            .unwrap_or(true)
                    }),
            );

            merge_adjacent_schema_completion_items(
                cursor,
                _keys,
                _accessors,
                current_schema,
                _schema_context,
                completion_hint,
                completion_items,
                self.one_of.as_deref(),
                self.any_of.as_deref(),
                self.all_of.as_deref(),
            )
            .await
        }
        .boxed()
    }
}

pub fn type_hint_string(
    offset: tombi_text::Offset,
    schema_base_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    [
        ("\"", "BasicString"),
        ("'", "LiteralString"),
        ("\"\"\"", "MultiLineBasicString"),
        ("'''", "MultiLineLiteralString"),
    ]
    .into_iter()
    .map(|(quote, detail)| {
        CompletionContent::new_type_hint_string(
            CompletionKind::String,
            quote,
            detail,
            CompletionEdit::new_string_literal(quote, offset, completion_hint),
            schema_base_uri,
        )
    })
    .collect()
}
