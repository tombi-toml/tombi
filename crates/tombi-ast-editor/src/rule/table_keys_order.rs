use std::borrow::Cow;

use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_comment_directive::value::{
    TableCommonFormatRules, TableCommonLintRules, TombiValueDirectiveContent,
};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, OneOfSchema, SchemaContext, TableSchema,
    ValueSchema, XTombiTableKeysOrder,
};
use tombi_syntax::SyntaxElement;
use tombi_validator::Validate;
use tombi_x_keyword::{TableKeysOrder, TableKeysOrderGroupKind};

#[derive(Debug, Clone, Copy)]
pub struct TableOrderOverride {
    pub disabled: bool,
    pub order: Option<TableKeysOrder>,
}

pub type TableOrderOverrides = tombi_hashmap::HashMap<Vec<Accessor>, TableOrderOverride>;

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

    if !schema_context.schema_table_keys_order_enabled(current_schema) {
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
        None,
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
    table_order_overrides: Option<&'b TableOrderOverrides>,
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
                    if let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
                        schemas,
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        schema_context.store,
                        &schema_context.schema_visits,
                        accessors,
                    )
                    .await
                    {
                        for current_schema in &resolved_schemas {
                            if value
                                .validate(accessors, Some(current_schema), schema_context)
                                .await
                                .is_ok()
                            {
                                return get_sorted_accessors(
                                    value,
                                    accessors,
                                    targets.clone(),
                                    Some(current_schema),
                                    schema_context,
                                    order.or(*keys_order),
                                    table_order_overrides,
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
        let mut sort_targets_map = tombi_hashmap::IndexMap::new();

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
                let table_override =
                    table_order_overrides.and_then(|overrides| overrides.get(accessors));
                let table_order_override = table_override.and_then(|override_order| {
                    if override_order.disabled {
                        None
                    } else {
                        override_order.order
                    }
                });
                let table_order =
                    get_table_keys_order(table_order_override.or(order), current_schema);
                let table_schema = current_schema.and_then(|current_schema| {
                    if let ValueSchema::Table(table_schema) = current_schema.value_schema.as_ref() {
                        Some(table_schema)
                    } else {
                        None
                    }
                });

                let sorted_targets = if table_override
                    .map(|override_order| override_order.disabled)
                    .unwrap_or(false)
                {
                    sort_targets_map.into_iter().collect_vec()
                } else {
                    sort_table_targets(sort_targets_map, table_schema, table_order.as_ref()).await
                };

                for (accessor, targets) in sorted_targets {
                    if let Some(value) = table.get(&accessor.to_string())
                        && let (Some(current_schema), Some(table_schema)) =
                            (current_schema, table_schema)
                    {
                        if let Ok(Some(current_schema)) = table_schema
                            .resolve_property_schema(
                                &tombi_schema_store::SchemaAccessor::from(&accessor),
                                current_schema.schema_uri.clone(),
                                current_schema.definitions.clone(),
                                schema_context.store,
                            )
                            .await
                            .inspect_err(|err| log::warn!("{err}"))
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
                                    table_order_overrides,
                                )
                                .await?,
                            );
                            continue;
                        }
                        if let Some((_, referable_schema)) =
                            &table_schema.additional_property_schema
                            && let Ok(Some(current_schema)) =
                                tombi_schema_store::resolve_schema_item(
                                    referable_schema,
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                                .inspect_err(|err| log::warn!("{err}"))
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
                                    table_order_overrides,
                                )
                                .await?,
                            );
                            continue;
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
                            table_order_overrides,
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
                if let Some(current_schema) = current_schema
                    && let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref()
                    && let Some(referable_schema) = &array_schema.items
                    && let Ok(Some(current_schema)) = tombi_schema_store::resolve_schema_item(
                        referable_schema,
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                    )
                    .await
                    .inspect_err(|err| log::warn!("{err}"))
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
                                table_order_overrides,
                            )
                            .await?,
                        );
                    }
                    return Some(results);
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
                            table_order_overrides,
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
                log::debug!("Table schema is not available, skipping schema sort");
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

fn get_table_keys_order(
    order: Option<TableKeysOrder>,
    current_schema: Option<&CurrentSchema>,
) -> Option<XTombiTableKeysOrder> {
    match order {
        Some(order) => Some(XTombiTableKeysOrder::All(order)),
        None => {
            if let Some(current_schema) = current_schema
                && let ValueSchema::Table(table_schema) = current_schema.value_schema.as_ref()
            {
                return table_schema.keys_order.clone();
            }
            None
        }
    }
}

async fn sort_table_targets<T>(
    sort_targets_map: tombi_hashmap::IndexMap<Accessor, Vec<(Vec<Accessor>, T)>>,
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
            .await;
        }
        (Some(XTombiTableKeysOrder::Groups(groups)), Some(table_schema)) => {
            let (mut has_keys_group, mut has_pattern_group, mut has_additional_group) =
                (false, false, false);
            for group in groups.iter() {
                match group.target {
                    TableKeysOrderGroupKind::Keys => has_keys_group = true,
                    TableKeysOrderGroupKind::PatternKeys => has_pattern_group = true,
                    TableKeysOrderGroupKind::AdditionalKeys => has_additional_group = true,
                }
            }
            // When no explicit AdditionalKeys group is specified, infer a sort
            // order from the first group's order so additional keys are still sorted.
            let fallback_additional_order = if has_additional_group {
                None
            } else {
                groups.first().and_then(|group| match group.order {
                    TableKeysOrder::Ascending
                    | TableKeysOrder::Descending
                    | TableKeysOrder::VersionSort => Some(group.order),
                    TableKeysOrder::Schema => None,
                })
            };
            let property_accessors: tombi_hashmap::HashSet<_> =
                table_schema.accessors().await.into_iter().collect();

            let mut pattern_regexes = Vec::new();
            if let Some(pattern_properties) = &table_schema.pattern_properties {
                for pattern_key in pattern_properties.read().await.keys() {
                    match tombi_regex::Regex::new(pattern_key) {
                        Ok(pattern) => pattern_regexes.push(pattern),
                        Err(_) => {
                            log::warn!("Invalid regex pattern property: {}", pattern_key);
                        }
                    }
                }
            }

            let mut original_slots = Vec::with_capacity(sort_targets_map.len());
            let mut unspecified_targets = Vec::new();

            let mut properties = Vec::new();
            let mut pattern_properties = Vec::new();
            let mut additional_properties = Vec::new();

            for (accessor, targets) in sort_targets_map {
                let kind = if property_accessors.contains(&accessor) {
                    TableKeysOrderGroupKind::Keys
                } else if accessor
                    .as_key()
                    .is_some_and(|key| pattern_regexes.iter().any(|pattern| pattern.is_match(key)))
                {
                    TableKeysOrderGroupKind::PatternKeys
                } else {
                    TableKeysOrderGroupKind::AdditionalKeys
                };

                let is_in_sort_group = match kind {
                    TableKeysOrderGroupKind::Keys => has_keys_group,
                    TableKeysOrderGroupKind::PatternKeys => has_pattern_group,
                    TableKeysOrderGroupKind::AdditionalKeys => {
                        has_additional_group || fallback_additional_order.is_some()
                    }
                };

                if is_in_sort_group {
                    original_slots.push(true);
                    match kind {
                        TableKeysOrderGroupKind::Keys => properties.push((accessor, targets)),
                        TableKeysOrderGroupKind::PatternKeys => {
                            pattern_properties.push((accessor, targets));
                        }
                        TableKeysOrderGroupKind::AdditionalKeys => {
                            additional_properties.push((accessor, targets));
                        }
                    }
                } else {
                    original_slots.push(false);
                    unspecified_targets.push((accessor, targets));
                }
            }

            let mut sorted_specified_targets = Vec::new();

            for group in groups {
                match group.target {
                    TableKeysOrderGroupKind::Keys => {
                        properties =
                            sort_targets(properties, group.order, Some(table_schema)).await;
                        sorted_specified_targets.append(&mut properties);
                    }
                    TableKeysOrderGroupKind::PatternKeys => {
                        pattern_properties =
                            sort_targets(pattern_properties, group.order, Some(table_schema)).await;
                        sorted_specified_targets.append(&mut pattern_properties);
                    }
                    TableKeysOrderGroupKind::AdditionalKeys => {
                        additional_properties =
                            sort_targets(additional_properties, group.order, Some(table_schema))
                                .await;
                        sorted_specified_targets.append(&mut additional_properties);
                    }
                }
            }

            if let Some(order) = fallback_additional_order {
                additional_properties =
                    sort_targets(additional_properties, order, Some(table_schema)).await;

                // Descending: specified keys first (Z→A), then additional keys (Z→A).
                // Ascending/VersionSort: additional keys first (A→Z), then specified keys.
                if matches!(order, TableKeysOrder::Descending) {
                    sorted_specified_targets.append(&mut additional_properties);
                } else {
                    additional_properties.append(&mut sorted_specified_targets);
                    sorted_specified_targets = additional_properties;
                }
            }

            let mut sorted_targets = Vec::with_capacity(original_slots.len());
            let mut sorted_specified_iter = sorted_specified_targets.into_iter();
            let mut unspecified_iter = unspecified_targets.into_iter();

            // Keep keys in unspecified groups at their original positions.
            for is_specified_slot in original_slots {
                let next = if is_specified_slot {
                    sorted_specified_iter.next()
                } else {
                    unspecified_iter.next()
                };
                if let Some(target) = next {
                    sorted_targets.push(target);
                }
            }

            sorted_targets.extend(sorted_specified_iter);
            sorted_targets.extend(unspecified_iter);

            return sorted_targets;
        }
        _ => {}
    }

    sort_targets_map.into_iter().collect_vec()
}
