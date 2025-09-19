use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_comment_directive::value::{
    TableCommonFormatRules, TableCommonLintRules, TombiValueDirectiveContent,
};
use tombi_schema_store::{CurrentSchema, SchemaContext};
use tombi_syntax::SyntaxElement;

use crate::rule::{inline_table_comma_trailing_comment, table_keys_order::get_sorted_accessors};

pub async fn inline_table_keys_order<'a>(
    value: &'a tombi_document_tree::Value,
    key_values_with_comma: Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    comment_directive: Option<
        TombiValueDirectiveContent<TableCommonFormatRules, TableCommonLintRules>,
    >,
) -> Vec<crate::Change> {
    if key_values_with_comma.is_empty() {
        return Vec::with_capacity(0);
    }

    if comment_directive
        .as_ref()
        .and_then(|c| c.table_keys_order_disabled())
        .unwrap_or(false)
    {
        return Vec::with_capacity(0);
    }

    let order = comment_directive
        .as_ref()
        .and_then(|comment_directive| comment_directive.table_keys_order().map(Into::into));

    let mut changes = vec![];

    let is_last_comma = key_values_with_comma
        .last()
        .map(|(_, comma)| comma.is_some())
        .unwrap_or(false);

    let old = std::ops::RangeInclusive::new(
        SyntaxElement::Node(key_values_with_comma.first().unwrap().0.syntax().clone()),
        SyntaxElement::Node(key_values_with_comma.last().unwrap().0.syntax().clone()),
    );

    let Some(mut sorted_key_values_with_comma) = get_sorted_accessors(
        value,
        &[],
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
    )
    .await
    else {
        return Vec::with_capacity(0);
    };

    if let Some((_, comma)) = sorted_key_values_with_comma.last_mut() {
        if !is_last_comma {
            if let Some(new_last_comma) = comma {
                if new_last_comma.trailing_comment().is_none()
                    && new_last_comma.leading_comments().next().is_none()
                {
                    *comma = None;
                }
            }
        }
    }

    for (value, comma) in &sorted_key_values_with_comma {
        changes.extend(inline_table_comma_trailing_comment(value, comma.as_ref()));
    }

    let new = sorted_key_values_with_comma
        .iter()
        .flat_map(|(value, comma)| {
            if let Some(comma) = comma {
                if !is_last_comma
                    && comma.leading_comments().next().is_none()
                    && comma.trailing_comment().is_none()
                {
                    vec![SyntaxElement::Node(value.syntax().clone())]
                } else {
                    vec![
                        SyntaxElement::Node(value.syntax().clone()),
                        SyntaxElement::Node(comma.syntax().clone()),
                    ]
                }
            } else {
                vec![SyntaxElement::Node(value.syntax().clone())]
            }
        })
        .collect_vec();

    changes.insert(0, crate::Change::ReplaceRange { old, new });

    changes
}
