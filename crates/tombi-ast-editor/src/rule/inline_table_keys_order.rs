use std::{cmp::Reverse, collections::HashSet};

use crate::rule::inline_table_comma_trailing_comment;
use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_schema_store::{
    Accessor, GroupTableKeysOrder, SchemaAccessor, SchemaContext, TableOrderSchema, TableSchema,
};
use tombi_syntax::SyntaxElement;
use tombi_version_sort::version_sort;
use tombi_x_keyword::{TableGroup, TableKeysOrder};

pub async fn inline_table_keys_order<'a>(
    mut key_values_with_comma: Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)>,
    table_schema: &'a TableSchema,
    schema_context: &'a SchemaContext<'a>,
) -> Vec<crate::Change> {
    if key_values_with_comma.is_empty() {
        return Vec::with_capacity(0);
    }

    let Some(keys_order) = &table_schema.keys_order else {
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
        TableOrderSchema::All(order) => {
            sort_targets(
                key_values_with_comma.into_iter().collect_vec(),
                *order,
                schema_context,
                &table_schema,
            )
            .await
        }
        TableOrderSchema::Groups(groups) => {
            let mut sorted_targets = Vec::with_capacity(key_values_with_comma.len());

            let mut properties = if has_group(groups, TableGroup::Properties) {
                extract_properties(&mut key_values_with_comma, &table_schema).await
            } else {
                Vec::new()
            };
            let mut pattern_properties = if has_group(groups, TableGroup::PatternProperties) {
                extract_pattern_properties(&mut key_values_with_comma, &table_schema).await
            } else {
                Vec::new()
            };
            let mut additional_properties = key_values_with_comma.into_iter().collect_vec();

            for group in groups {
                match group.target {
                    TableGroup::Properties => {
                        properties =
                            sort_targets(properties, group.order, schema_context, &table_schema)
                                .await;
                        sorted_targets.append(&mut properties);
                    }
                    TableGroup::PatternProperties => {
                        pattern_properties = sort_targets(
                            pattern_properties,
                            group.order,
                            schema_context,
                            &table_schema,
                        )
                        .await;
                        sorted_targets.append(&mut pattern_properties);
                    }
                    TableGroup::AdditionalProperties => {
                        additional_properties = sort_targets(
                            additional_properties,
                            group.order,
                            schema_context,
                            &table_schema,
                        )
                        .await;
                        sorted_targets.append(&mut additional_properties);
                    }
                }
            }
            sorted_targets
        }
    };

    if let Some((_, comma)) = sorted_key_values_with_comma.last_mut() {
        if !is_last_comma {
            if let Some(last_comma) = comma {
                if last_comma.trailing_comment().is_none()
                    && last_comma.leading_comments().next().is_none()
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
                if comma.trailing_comment().is_none() && comma.leading_comments().next().is_none() {
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

/// Extracts the properties, and sorts them by the schema
async fn extract_properties(
    key_values_with_comma: &mut Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)>,
    table_schema: &TableSchema,
) -> Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)> {
    let accessors: HashSet<_> = table_schema.accessors().await.into_iter().collect();

    key_values_with_comma
        .extract_if(.., |(element, _)| {
            if let Some(keys) = &element.keys() {
                accessors.contains(&Accessor::Key(keys.to_string()))
            } else {
                false
            }
        })
        .collect()
}

/// Extracts the pattern properties, and sorts them by the schema
async fn extract_pattern_properties(
    key_values_with_comma: &mut Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)>,
    table_schema: &TableSchema,
) -> Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)> {
    let mut sorted_targets = vec![];
    let Some(pattern_properties) = &table_schema.pattern_properties else {
        return sorted_targets;
    };
    for (pattern_key, ..) in pattern_properties.write().await.iter_mut() {
        let Ok(pattern) = regex::Regex::new(pattern_key) else {
            tracing::warn!("Invalid regex pattern property: {}", pattern_key);
            continue;
        };
        sorted_targets.extend(key_values_with_comma.extract_if(.., |(key_value, _)| {
            key_value
                .keys()
                .as_ref()
                .map(ToString::to_string)
                .map(|key| pattern.is_match(&key))
                .unwrap_or_default()
        }));
    }
    sorted_targets
}

async fn sort_targets<'a>(
    mut key_values_with_comma: Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)>,
    order: TableKeysOrder,
    schema_context: &'a SchemaContext<'a>,
    table_schema: &TableSchema,
) -> Vec<(tombi_ast::KeyValue, Option<tombi_ast::Comma>)> {
    match order {
        TableKeysOrder::Ascending => key_values_with_comma.sort_by_key(|(key, _)| {
            key.keys()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .try_to_raw_text(schema_context.toml_version)
                .unwrap()
        }),
        TableKeysOrder::Descending => key_values_with_comma.sort_by_key(|(key, _)| {
            Reverse(
                key.keys()
                    .unwrap()
                    .keys()
                    .next()
                    .unwrap()
                    .try_to_raw_text(schema_context.toml_version)
                    .unwrap(),
            )
        }),
        TableKeysOrder::Schema => {
            let mut new_key_values_with_comma = vec![];
            for (schema_accessor, _) in table_schema.properties.write().await.iter_mut() {
                new_key_values_with_comma.extend(key_values_with_comma.extract_if(
                    ..,
                    |(element, ..)| {
                        if let Some(keys) = &element.keys() {
                            schema_accessor == &SchemaAccessor::Key(keys.to_string())
                        } else {
                            false
                        }
                    },
                ));
            }

            new_key_values_with_comma.append(&mut key_values_with_comma);
            return new_key_values_with_comma;
        }
        TableKeysOrder::VersionSort => key_values_with_comma.sort_by(|(a, _), (b, _)| {
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
        }),
    };
    key_values_with_comma
}

fn has_group(sort_groups: &[GroupTableKeysOrder], group: TableGroup) -> bool {
    sort_groups.iter().any(|g| g.target == group)
}
