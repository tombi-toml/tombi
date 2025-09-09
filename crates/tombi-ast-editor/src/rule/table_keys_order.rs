use std::{borrow::Cow, collections::HashSet};

use indexmap::IndexMap;
use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, GroupTableKeysOrder, OneOfSchema,
    PropertySchema, SchemaContext, TableKeysOrderSpec, TableSchema, ValueSchema,
};
use tombi_syntax::SyntaxElement;
use tombi_validator::Validate;
use tombi_x_keyword::{TableKeysOrder, TableKeysOrderGroup};

pub async fn table_keys_order<'a>(
    value: &'a tombi_document_tree::Value,
    key_values: Vec<tombi_ast::KeyValue>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
) -> Vec<crate::Change> {
    if key_values.is_empty() {
        return Vec::with_capacity(0);
    }

    let old = std::ops::RangeInclusive::new(
        SyntaxElement::Node(key_values.first().unwrap().syntax().clone()),
        SyntaxElement::Node(key_values.last().unwrap().syntax().clone()),
    );

    let targets = key_values
        .into_iter()
        .map(|kv| {
            (
                kv.keys()
                    .map(|key| {
                        key.keys()
                            .map(|key| Accessor::Key(key.to_raw_text(schema_context.toml_version)))
                            .collect_vec()
                    })
                    .unwrap_or_default(),
                kv,
            )
        })
        .collect_vec();

    let new = sorted_accessors(value, &[], targets, current_schema, schema_context)
        .await
        .into_iter()
        .map(|kv| SyntaxElement::Node(kv.syntax().clone()))
        .collect_vec();

    vec![crate::Change::ReplaceRange { old, new }]
}

pub fn sorted_accessors<'a: 'b, 'b, T>(
    value: &'a tombi_document_tree::Value,
    accessors: &'a [tombi_schema_store::Accessor],
    targets: Vec<(Vec<tombi_schema_store::Accessor>, T)>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
) -> BoxFuture<'b, Vec<T>>
where
    T: Send + Clone + std::fmt::Debug + 'b,
{
    async move {
        if let Some(CurrentSchema {
            value_schema,
            schema_uri,
            definitions,
        }) = current_schema
        {
            match value_schema.as_ref() {
                ValueSchema::OneOf(OneOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                | ValueSchema::AllOf(AllOfSchema { schemas, .. }) => {
                    for schema in schemas.write().await.iter_mut() {
                        if let Ok(Some(current_schema)) = schema
                            .resolve(
                                Cow::Borrowed(schema_uri),
                                Cow::Borrowed(definitions),
                                schema_context.store,
                            )
                            .await
                            .inspect_err(|err| tracing::warn!("{err}"))
                        {
                            if value
                                .validate(accessors, Some(&current_schema), schema_context)
                                .await
                                .is_ok()
                            {
                                return sorted_accessors(
                                    value,
                                    accessors,
                                    targets.clone(),
                                    Some(&current_schema),
                                    schema_context,
                                )
                                .await;
                            }
                        }
                    }
                    return targets.into_iter().map(|(_, target)| target).collect_vec();
                }
                _ => {}
            }
        }

        let mut results = Vec::with_capacity(targets.len());
        let mut new_targets_map = IndexMap::new();
        for (schema_accessors, target) in targets {
            if let Some(accessor) = schema_accessors.first() {
                new_targets_map
                    .entry(accessor.clone())
                    .or_insert_with(Vec::new)
                    .push((schema_accessors[1..].to_vec(), target));
            } else {
                results.push(target);
            }
        }

        if let Some(current_schema) = current_schema {
            match (value, current_schema.value_schema.as_ref()) {
                (tombi_document_tree::Value::Table(table), ValueSchema::Table(table_schema)) => {
                    if new_targets_map
                        .iter()
                        .all(|(accessor, _)| matches!(accessor, Accessor::Key(_)))
                    {
                        let sorted_targets = match &table_schema.keys_order {
                            Some(TableKeysOrderSpec::All(order)) => {
                                sort_targets(
                                    new_targets_map.into_iter().collect_vec(),
                                    *order,
                                    table_schema,
                                )
                                .await
                            }
                            Some(TableKeysOrderSpec::Groups(groups)) => {
                                let mut sorted_targets = Vec::with_capacity(new_targets_map.len());

                                let mut properties = if has_group(groups, TableKeysOrderGroup::Keys)
                                {
                                    extract_properties(&mut new_targets_map, table_schema).await
                                } else {
                                    Vec::new()
                                };
                                let mut pattern_properties =
                                    if has_group(groups, TableKeysOrderGroup::PatternKeys) {
                                        extract_pattern_properties(
                                            &mut new_targets_map,
                                            table_schema,
                                        )
                                        .await
                                    } else {
                                        Vec::new()
                                    };
                                let mut additional_properties =
                                    new_targets_map.into_iter().collect_vec();

                                for group in groups {
                                    match group.target {
                                        TableKeysOrderGroup::Keys => {
                                            properties =
                                                sort_targets(properties, group.order, table_schema)
                                                    .await;
                                            sorted_targets.append(&mut properties);
                                        }
                                        TableKeysOrderGroup::PatternKeys => {
                                            pattern_properties = sort_targets(
                                                pattern_properties,
                                                group.order,
                                                table_schema,
                                            )
                                            .await;
                                            sorted_targets.append(&mut pattern_properties);
                                        }
                                        TableKeysOrderGroup::AdditionalKeys => {
                                            additional_properties = sort_targets(
                                                additional_properties,
                                                group.order,
                                                table_schema,
                                            )
                                            .await;
                                            sorted_targets.append(&mut additional_properties);
                                        }
                                    }
                                }
                                sorted_targets
                            }
                            None => new_targets_map.into_iter().collect_vec(),
                        };

                        for (accessor, targets) in sorted_targets {
                            if let Some(value) = table.get(&accessor.to_string()) {
                                if let Some(PropertySchema {
                                    property_schema, ..
                                }) = table_schema.properties.write().await.get_mut(&accessor)
                                {
                                    if let Ok(Some(current_schema)) = property_schema
                                        .resolve(
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                        .inspect_err(|err| tracing::warn!("{err}"))
                                    {
                                        results.extend(
                                            sorted_accessors(
                                                value,
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor))
                                                    .collect_vec(),
                                                targets,
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await,
                                        );
                                    } else {
                                        results
                                            .extend(targets.into_iter().map(|(_, target)| target));
                                    }
                                } else if let Some((_, referable_schema)) =
                                    &table_schema.additional_property_schema
                                {
                                    if let Ok(Some(current_schema)) = referable_schema
                                        .write()
                                        .await
                                        .resolve(
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                        .inspect_err(|err| tracing::warn!("{err}"))
                                    {
                                        results.extend(
                                            sorted_accessors(
                                                value,
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor))
                                                    .collect_vec(),
                                                targets,
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await,
                                        );
                                    } else {
                                        results
                                            .extend(targets.into_iter().map(|(_, target)| target));
                                    }
                                } else {
                                    results.extend(targets.into_iter().map(|(_, target)| target));
                                }
                            } else {
                                results.extend(targets.into_iter().map(|(_, target)| target));
                            }
                        }
                        return results;
                    }
                }
                (tombi_document_tree::Value::Array(array), ValueSchema::Array(array_schema)) => {
                    if new_targets_map
                        .iter()
                        .all(|(accessor, _)| matches!(accessor, Accessor::Index(_)))
                    {
                        if let Some(referable_schema) = &array_schema.items {
                            if let Ok(Some(current_schema)) = referable_schema
                                .write()
                                .await
                                .resolve(
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                                .inspect_err(|err| tracing::warn!("{err}"))
                            {
                                for (value, (_, targets)) in array.iter().zip(new_targets_map) {
                                    results.extend(
                                        sorted_accessors(
                                            value,
                                            accessors,
                                            targets,
                                            Some(&current_schema),
                                            schema_context,
                                        )
                                        .await,
                                    );
                                }
                            } else {
                                for targets in
                                    new_targets_map.into_iter().map(|(_, targets)| targets)
                                {
                                    results.extend(targets.into_iter().map(|(_, target)| target));
                                }
                            }
                        }

                        return results;
                    }
                }
                _ => {}
            }
        }

        for (_, targets) in new_targets_map {
            results.extend(targets.into_iter().map(|(_, target)| target));
        }

        results
    }
    .boxed()
}

/// Extracts the properties, and sorts them by the schema
async fn extract_properties<T>(
    targets_map: &mut IndexMap<Accessor, Vec<(Vec<Accessor>, T)>>,
    table_schema: &TableSchema,
) -> Vec<(Accessor, Vec<(Vec<Accessor>, T)>)> {
    let accessors: HashSet<_> = table_schema.accessors().await.into_iter().collect();
    targets_map
        .extract_if(.., |element, _| accessors.contains(element))
        .collect()
}

/// Extracts the pattern properties, and sorts them by the schema
async fn extract_pattern_properties<T>(
    targets_map: &mut IndexMap<Accessor, Vec<(Vec<Accessor>, T)>>,
    table_schema: &TableSchema,
) -> Vec<(Accessor, Vec<(Vec<Accessor>, T)>)> {
    let mut sorted_targets = vec![];
    let Some(pattern_properties) = &table_schema.pattern_properties else {
        return sorted_targets;
    };
    for (pattern_key, ..) in pattern_properties.write().await.iter_mut() {
        let Ok(pattern) = regex::Regex::new(pattern_key) else {
            tracing::warn!("Invalid regex pattern property: {}", pattern_key);
            continue;
        };
        sorted_targets.extend(targets_map.extract_if(.., |key, _| {
            key.as_key()
                .map(|key| pattern.is_match(key))
                .unwrap_or_default()
        }));
    }
    sorted_targets
}

async fn sort_targets<T>(
    mut targets: Vec<(Accessor, Vec<(Vec<Accessor>, T)>)>,
    order: TableKeysOrder,
    table_schema: &TableSchema,
) -> Vec<(Accessor, Vec<(Vec<Accessor>, T)>)> {
    match order {
        TableKeysOrder::Ascending => targets.sort_by(|(a_accessor, _), (b_accessor, _)| {
            a_accessor.partial_cmp(b_accessor).unwrap()
        }),
        TableKeysOrder::Descending => targets.sort_by(|(a_accessor, _), (b_accessor, _)| {
            b_accessor.partial_cmp(a_accessor).unwrap()
        }),
        TableKeysOrder::Schema => {
            let mut new_targets = vec![];
            for accessor in table_schema.accessors().await {
                new_targets.extend(targets.extract_if(.., |(element, ..)| *element == accessor));
            }
            new_targets.append(&mut targets);
            return new_targets;
        }
        TableKeysOrder::VersionSort => {
            targets.sort_by(
                |(a_accessor, _), (b_accessor, _)| match (a_accessor, b_accessor) {
                    (Accessor::Key(a_key), Accessor::Key(b_key)) => {
                        tombi_version_sort::version_sort(a_key, b_key)
                    }
                    _ => unreachable!("Unexpected accessor type in table keys order sorting"),
                },
            );
        }
    };
    targets
}

fn has_group(sort_groups: &[GroupTableKeysOrder], group: TableKeysOrderGroup) -> bool {
    sort_groups.iter().any(|g| g.target == group)
}
