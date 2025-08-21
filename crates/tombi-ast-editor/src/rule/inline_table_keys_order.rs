use crate::rule::inline_table_comma_trailing_comment;
use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_schema_store::{SchemaContext, TableSchema};
use tombi_syntax::SyntaxElement;
use tombi_version_sort::version_sort;
use tombi_x_keyword::TableKeysOrder;

pub async fn inline_table_keys_order<'a>(
    key_values_with_comma: Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)>,
    table_schema: &'a TableSchema,
    schema_context: &'a SchemaContext<'a>,
) -> Vec<crate::Change> {
    if key_values_with_comma.is_empty() {
        return Vec::with_capacity(0);
    }

    let Some(keys_order) = table_schema.keys_order else {
        return Vec::with_capacity(0);
    };

    let mut changes = vec![];

    let is_last_comma = key_values_with_comma
        .last()
        .map(|(_, comma)| comma.is_some())
        .unwrap_or(false);

    let old = std::ops::RangeInclusive::new(
        SyntaxElement::Node(key_values_with_comma.first().unwrap().0.syntax().clone()),
        SyntaxElement::Node(key_values_with_comma.last().unwrap().0.syntax().clone()),
    );

    let mut sorted_key_values_with_comma = match keys_order {
        TableKeysOrder::Ascending => key_values_with_comma
            .into_iter()
            .sorted_by_key(|(key, _)| {
                key.keys()
                    .unwrap()
                    .keys()
                    .next()
                    .unwrap()
                    .try_to_raw_text(schema_context.toml_version)
                    .unwrap()
            })
            .collect_vec(),
        TableKeysOrder::Descending => key_values_with_comma
            .into_iter()
            .rev()
            .sorted_by_key(|(key, _)| {
                key.keys()
                    .unwrap()
                    .keys()
                    .next()
                    .unwrap()
                    .try_to_raw_text(schema_context.toml_version)
                    .unwrap()
            })
            .collect_vec(),
        TableKeysOrder::Schema => {
            let mut new_key_values_with_comma = vec![];
            let mut key_values_with_comma = key_values_with_comma;
            for (accessor, _) in table_schema.properties.write().await.iter_mut() {
                key_values_with_comma = key_values_with_comma
                    .into_iter()
                    .filter_map(|(key_value, comma)| {
                        if key_value.keys().iter().next().map(ToString::to_string)
                            == Some(accessor.to_string())
                        {
                            new_key_values_with_comma.push((key_value, comma));
                            None
                        } else {
                            Some((key_value, comma))
                        }
                    })
                    .collect_vec();
            }
            new_key_values_with_comma.extend(key_values_with_comma);

            new_key_values_with_comma
        }
        TableKeysOrder::VersionSort => key_values_with_comma
            .into_iter()
            .sorted_by(|(a, _), (b, _)| {
                let a_key = a
                    .keys()
                    .unwrap()
                    .keys()
                    .next()
                    .unwrap()
                    .try_to_raw_text(schema_context.toml_version)
                    .unwrap();
                let b_key = b
                    .keys()
                    .unwrap()
                    .keys()
                    .next()
                    .unwrap()
                    .try_to_raw_text(schema_context.toml_version)
                    .unwrap();
                version_sort(&a_key, &b_key)
            })
            .collect_vec(),
    };

    if let Some((_, comma)) = sorted_key_values_with_comma.last_mut() {
        if !is_last_comma {
            if let Some(last_comma) = comma {
                if last_comma.trailing_comment().is_none()
                    && last_comma.leading_comments().collect_vec().is_empty()
                {
                    *comma = None;
                }
            }
        }
    }

    for (key_value, comma) in &sorted_key_values_with_comma {
        changes.extend(inline_table_comma_trailing_comment(
            key_value,
            comma.as_ref(),
        ));
    }

    let new = sorted_key_values_with_comma
        .iter()
        .flat_map(|(key_value, comma)| {
            if let Some(comma) = comma {
                vec![
                    SyntaxElement::Node(key_value.syntax().clone()),
                    SyntaxElement::Node(comma.syntax().clone()),
                ]
            } else {
                vec![SyntaxElement::Node(key_value.syntax().clone())]
            }
        })
        .collect_vec();

    if !is_last_comma {
        if let Some(tombi_syntax::SyntaxElement::Node(node)) = new.last() {
            if let Some(comma) = tombi_ast::Comma::cast(node.clone()) {
                if comma.trailing_comment().is_none()
                    && comma.leading_comments().collect_vec().is_empty()
                {
                    changes.push(crate::Change::Remove {
                        target: SyntaxElement::Node(comma.syntax().clone()),
                    });
                }
            }
        }
    }

    changes.insert(0, crate::Change::ReplaceRange { old, new });

    changes
}
