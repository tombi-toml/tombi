use ahash::HashSet;
use std::borrow::Cow;

use indexmap::IndexMap;
use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_comment_directive::value::{
    TableCommonFormatRules, TableCommonLintRules, TombiValueDirectiveContent,
};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, OneOfSchema, PropertySchema, SchemaContext,
    TableKeysOrderGroup, TableSchema, ValueSchema, XTombiTableKeysOrder,
};
use tombi_syntax::SyntaxElement;
use tombi_validator::Validate;
use tombi_x_keyword::{TableKeysOrder, TableKeysOrderGroupKind};

pub async fn table_keys_order<'a>(
    value: &'a tombi_document_tree::Value,
    key_values: Vec<tombi_ast::KeyValue>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    comment_directive: Option<
        TombiValueDirectiveContent<TableCommonFormatRules, TableCommonLintRules>,
    >,
) -> Vec<crate::Change> {
    if key_values.is_empty() {
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

    let old = std::ops::RangeInclusive::new(
        SyntaxElement::Node(key_values.first().unwrap().syntax().clone()),
        SyntaxElement::Node(key_values.last().unwrap().syntax().clone()),
    );

    let Some(sorted_key_values) = get_sorted_accessors(
        value,
        &[],
        key_values
            .into_iter()
            .map(|kv| {
                (
                    kv.get_accessors(schema_context.toml_version)
                        .unwrap_or_default(),
                    kv,
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

    let new = sorted_key_values
        .into_iter()
        .map(|kv| SyntaxElement::Node(kv.syntax().clone()))
        .collect_vec();

    vec![crate::Change::ReplaceRange { old, new }]
}

pub fn get_sorted_accessors<'a: 'b, 'b, T>(
    value: &'a tombi_document_tree::Value,
    accessors: &'a [tombi_schema_store::Accessor],
    targets: Vec<(Vec<tombi_schema_store::Accessor>, T)>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    order: Option<TableKeysOrder>,
) -> BoxFuture<'b, Option<Vec<T>>>
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
                ValueSchema::OneOf(OneOfSchema {
                    schemas,
                    keys_order,
                    ..
                })
                | ValueSchema::AnyOf(AnyOfSchema {
                    schemas,
                    keys_order,
                    ..
                })
                | ValueSchema::AllOf(AllOfSchema {
                    schemas,
                    keys_order,
                    ..
                }) => {
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
                                return get_sorted_accessors(
                                    value,
                                    accessors,
                                    targets.clone(),
                                    Some(&current_schema),
                                    schema_context,
                                    order.or(*keys_order),
                                )
                                .await;
                            }
                        }
                    }
                    return None;
                }
                _ => {}
            }
        }

        let mut results = Vec::with_capacity(targets.len());
        let mut sort_targets_map = IndexMap::new();

        for (accessors, target) in targets {
            if let Some(accessor) = accessors.first() {
                sort_targets_map
                    .entry(accessor.clone())
                    .or_insert_with(Vec::new)
                    .push((accessors[1..].to_vec(), target));
            } else {
                results.push(target);
            }
        }

        match value {
            tombi_document_tree::Value::Table(table)
                if sort_targets_map
                    .iter()
                    .all(|(accessor, _)| matches!(accessor, Accessor::Key(_))) =>
            {
                let table_order = get_table_keys_order(order, current_schema);
                let table_schema = current_schema.and_then(|current_schema| {
                    if let ValueSchema::Table(table_schema) = current_schema.value_schema.as_ref() {
                        Some(table_schema)
                    } else {
                        None
                    }
                });

                let sorted_targets =
                    sort_table_targets(sort_targets_map, table_schema, table_order.as_ref()).await;

                for (accessor, targets) in sorted_targets {
                    if let Some(value) = table.get(&accessor.to_string()) {
                        if let (Some(current_schema), Some(table_schema)) =
                            (current_schema, table_schema)
                        {
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
                                        get_sorted_accessors(
                                            value,
                                            &accessors
                                                .iter()
                                                .cloned()
                                                .chain(std::iter::once(accessor))
                                                .collect_vec(),
                                            targets,
                                            Some(&current_schema),
                                            schema_context,
                                            order,
                                        )
                                        .await?,
                                    );
                                    continue;
                                }
                            }
                            if let Some((_, referable_schema)) =
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
                                        get_sorted_accessors(
                                            value,
                                            &accessors
                                                .iter()
                                                .cloned()
                                                .chain(std::iter::once(accessor))
                                                .collect_vec(),
                                            targets,
                                            Some(&current_schema),
                                            schema_context,
                                            order,
                                        )
                                        .await?,
                                    );
                                    continue;
                                }
                            }
                        }
                    }

                    results.extend(
                        get_sorted_accessors(
                            value,
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(accessor))
                                .collect_vec(),
                            targets,
                            None,
                            schema_context,
                            order,
                        )
                        .await?,
                    );
                }

                Some(results)
            }
            tombi_document_tree::Value::Array(array)
                if sort_targets_map
                    .iter()
                    .all(|(accessor, _)| matches!(accessor, Accessor::Index(_))) =>
            {
                if let Some(current_schema) = current_schema {
                    if let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref() {
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
                                for (index, (value, (_, targets))) in
                                    array.iter().zip(sort_targets_map).enumerate()
                                {
                                    results.extend(
                                        get_sorted_accessors(
                                            value,
                                            &accessors
                                                .iter()
                                                .cloned()
                                                .chain(std::iter::once(Accessor::Index(index)))
                                                .collect_vec(),
                                            targets,
                                            Some(&current_schema),
                                            schema_context,
                                            order,
                                        )
                                        .await?,
                                    );
                                }
                                return Some(results);
                            }
                        }
                    }
                };

                for (index, (value, (_, targets))) in array.iter().zip(sort_targets_map).enumerate()
                {
                    results.extend(
                        get_sorted_accessors(
                            value,
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(Accessor::Index(index)))
                                .collect_vec(),
                            targets,
                            None,
                            schema_context,
                            order,
                        )
                        .await?,
                    );
                }
                Some(results)
            }
            _ => {
                for (_, targets) in sort_targets_map {
                    results.extend(targets.into_iter().map(|(_, target)| target));
                }

                Some(results)
            }
        }
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

#[allow(clippy::type_complexity)]
async fn sort_targets<T>(
    mut targets: Vec<(Accessor, Vec<(Vec<Accessor>, T)>)>,
    order: TableKeysOrder,
    table_schema: Option<&TableSchema>,
) -> Vec<(Accessor, Vec<(Vec<Accessor>, T)>)> {
    match order {
        TableKeysOrder::Ascending => targets.sort_by(|(a_accessor, _), (b_accessor, _)| {
            a_accessor.partial_cmp(b_accessor).unwrap()
        }),
        TableKeysOrder::Descending => targets.sort_by(|(a_accessor, _), (b_accessor, _)| {
            b_accessor.partial_cmp(a_accessor).unwrap()
        }),
        TableKeysOrder::Schema => {
            let Some(table_schema) = table_schema else {
                tracing::debug!("Table schema is not available, skipping schema sort");
                return targets;
            };
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

fn has_group(sort_groups: &[TableKeysOrderGroup], group: TableKeysOrderGroupKind) -> bool {
    sort_groups.iter().any(|g| g.target == group)
}

fn get_table_keys_order(
    order: Option<TableKeysOrder>,
    current_schema: Option<&CurrentSchema>,
) -> Option<XTombiTableKeysOrder> {
    match order {
        Some(order) => Some(XTombiTableKeysOrder::All(order)),
        None => {
            if let Some(current_schema) = current_schema {
                if let ValueSchema::Table(table_schema) = current_schema.value_schema.as_ref() {
                    return table_schema.keys_order.clone();
                }
            }
            None
        }
    }
}

async fn sort_table_targets<T>(
    mut sort_targets_map: IndexMap<Accessor, Vec<(Vec<Accessor>, T)>>,
    table_schema: Option<&TableSchema>,
    order: Option<&XTombiTableKeysOrder>,
) -> Vec<(Accessor, Vec<(Vec<Accessor>, T)>)> {
    match (order, table_schema) {
        (Some(XTombiTableKeysOrder::All(order)), _) => {
            return sort_targets(
                sort_targets_map.into_iter().collect_vec(),
                *order,
                table_schema,
            )
            .await
        }
        (Some(XTombiTableKeysOrder::Groups(groups)), Some(table_schema)) => {
            let mut sorted_targets = Vec::with_capacity(sort_targets_map.len());

            let mut properties = if has_group(groups, TableKeysOrderGroupKind::Keys) {
                extract_properties(&mut sort_targets_map, table_schema).await
            } else {
                Vec::with_capacity(0)
            };
            let mut pattern_properties = if has_group(groups, TableKeysOrderGroupKind::PatternKeys)
            {
                extract_pattern_properties(&mut sort_targets_map, table_schema).await
            } else {
                Vec::with_capacity(0)
            };
            let mut additional_properties = sort_targets_map.into_iter().collect_vec();

            for group in groups {
                match group.target {
                    TableKeysOrderGroupKind::Keys => {
                        properties =
                            sort_targets(properties, group.order, Some(table_schema)).await;
                        sorted_targets.append(&mut properties);
                    }
                    TableKeysOrderGroupKind::PatternKeys => {
                        pattern_properties =
                            sort_targets(pattern_properties, group.order, Some(table_schema)).await;
                        sorted_targets.append(&mut pattern_properties);
                    }
                    TableKeysOrderGroupKind::AdditionalKeys => {
                        additional_properties =
                            sort_targets(additional_properties, group.order, Some(table_schema))
                                .await;
                        sorted_targets.append(&mut additional_properties);
                    }
                }
            }
            return sorted_targets;
        }
        _ => {}
    }

    sort_targets_map.into_iter().collect_vec()
}
