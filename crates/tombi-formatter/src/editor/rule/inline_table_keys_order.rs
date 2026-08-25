use itertools::Itertools;
use tombi_ast_syntax::AstNode;
use tombi_comment_directive::value::{
    TableCommonFormatRules, TableCommonLintRules, TombiValueDirectiveContent,
};
use tombi_schema_store::{CurrentSchema, SchemaContext};

use crate::editor::change::SourcePart;
use crate::editor::rule::{
    inline_table_comma_trailing_comment::inline_table_comma_trailing_comment,
    table_keys_order::get_sorted_accessors,
};

pub(in crate::editor) async fn inline_table_keys_order<'a>(
    node: &'a tombi_document_tree_syntax::Value,
    accessors: &'a [tombi_schema_store::Accessor],
    key_values_with_comma: Vec<(tombi_ast_syntax::KeyValue, Option<tombi_ast_syntax::Comma>)>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    comment_directive: Option<
        TombiValueDirectiveContent<TableCommonFormatRules, TableCommonLintRules>,
    >,
) -> Vec<crate::editor::Change> {
    if key_values_with_comma.is_empty() {
        return Vec::new();
    }

    if comment_directive
        .as_ref()
        .and_then(|c| c.table_keys_order_disabled())
        .unwrap_or_default()
    {
        return Vec::new();
    }

    let order = comment_directive
        .as_ref()
        .and_then(|comment_directive| comment_directive.table_keys_order().map(Into::into));

    let schema_override = schema_context.table_order_override(current_schema, accessors);
    if schema_override.is_some_and(|override_item| override_item.disabled) {
        return Vec::new();
    }

    let schema_order_enabled = schema_override.is_some_and(|override_item| !override_item.disabled)
        || schema_context.schema_table_keys_order_enabled(current_schema);
    if order.is_none() && !schema_order_enabled {
        return Vec::new();
    }

    let old_order = key_values_with_comma
        .iter()
        .map(|(key_value, _)| key_value.syntax().range())
        .collect_vec();
    let mut changes = vec![];

    let is_last_comma = key_values_with_comma
        .last()
        .map(|(_, comma)| comma.is_some())
        .unwrap_or_default();

    let old_first = key_values_with_comma.first().unwrap().0.syntax().clone();
    let old_last = key_values_with_comma.last().unwrap().0.syntax().clone();

    let Some(mut sorted_key_values_with_comma) = get_sorted_accessors(
        node,
        accessors,
        key_values_with_comma
            .into_iter()
            .map(|(kv, comma)| {
                (
                    kv.get_accessors(schema_context.toml_version)
                        .unwrap_or_default(),
                    (kv, comma),
                )
            })
            .collect_vec(),
        current_schema,
        schema_context,
        order,
        None,
    )
    .await
    else {
        return Vec::new();
    };

    if old_order.into_iter().eq(sorted_key_values_with_comma
        .iter()
        .map(|(key_value, _)| key_value.syntax().range()))
    {
        return Vec::new();
    }

    if let Some((_, comma)) = sorted_key_values_with_comma.last_mut()
        && !is_last_comma
        && let Some(new_last_comma) = comma
        && new_last_comma.trailing_comment().is_none()
        && new_last_comma.leading_comments().next().is_none()
    {
        *comma = None;
    }

    let sorted_len = sorted_key_values_with_comma.len();
    for (i, (value, comma)) in sorted_key_values_with_comma.iter().enumerate() {
        changes.extend(inline_table_comma_trailing_comment(
            value,
            comma.as_ref(),
            is_last_comma || i + 1 != sorted_len,
        ));
    }

    let mut new = Vec::with_capacity(sorted_len * 2);
    for (i, (value, comma)) in sorted_key_values_with_comma.iter().enumerate() {
        new.push(SourcePart::node(value));
        if let Some(comma) = comma
            && (is_last_comma
                || i + 1 != sorted_len
                || comma.leading_comments().next().is_some()
                || comma.trailing_comment().is_some())
        {
            new.push(SourcePart::node(comma));
        }
    }

    changes.insert(
        0,
        crate::editor::Change::replace_range(&old_first, &old_last, new),
    );

    changes
}
