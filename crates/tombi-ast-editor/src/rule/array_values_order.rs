mod boolean;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;

use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_comment_directive::value::{
    ArrayCommonFormatRules, ArrayCommonLintRules, TombiValueDirectiveContent,
};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, OneOfSchema, SchemaContext, TableSchema,
    ValueSchema, XTombiArrayValuesOrder,
};
use tombi_syntax::SyntaxElement;
use tombi_validator::Validate;
use tombi_x_keyword::{ArrayValuesOrder, ArrayValuesOrderBy, ArrayValuesOrderGroup};

use boolean::create_boolean_sortable_values;
use integer::create_integer_sortable_values;
use local_date::create_local_date_sortable_values;
use local_date_time::create_local_date_time_sortable_values;
use local_time::create_local_time_sortable_values;
use offset_date_time::create_offset_date_time_sortable_values;
use string::create_string_sortable_values;

use crate::rule::array_comma_trailing_comment;

pub async fn array_values_order<'a>(
    values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    array_node: &'a tombi_document_tree::Array,
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    array_schema_values_order: Option<XTombiArrayValuesOrder>,
    comment_directive: Option<
        TombiValueDirectiveContent<ArrayCommonFormatRules, ArrayCommonLintRules>,
    >,
) -> Vec<crate::Change> {
    if values_with_comma.is_empty() {
        return Vec::with_capacity(0);
    }

    if comment_directive
        .as_ref()
        .and_then(|c| c.array_values_order_disabled())
        .unwrap_or(false)
    {
        return Vec::with_capacity(0);
    }

    let order: Option<ArrayValuesOrder> = comment_directive
        .as_ref()
        .and_then(|comment_directive| comment_directive.array_values_order().map(Into::into));

    let values_order = match order {
        Some(values_order) => Some(XTombiArrayValuesOrder::All(values_order)),
        None => array_schema_values_order,
    };

    let Some(values_order) = values_order else {
        return Vec::with_capacity(0);
    };

    let mut changes = vec![];

    let is_last_comma = values_with_comma
        .last()
        .map(|(_, comma)| comma.is_some())
        .unwrap_or(false);

    let old = std::ops::RangeInclusive::new(
        SyntaxElement::Node(values_with_comma.first().unwrap().0.syntax().clone()),
        SyntaxElement::Node(values_with_comma.last().unwrap().0.syntax().clone()),
    );

    let sorted_values_with_comma = match values_order {
        XTombiArrayValuesOrder::All(values_order) => {
            get_sorted_values_order_all(
                values_with_comma,
                array_node,
                accessors,
                current_schema,
                schema_context,
                values_order,
            )
            .await
        }
        XTombiArrayValuesOrder::Groups(values_order_group) => {
            get_sorted_values_order_groups(
                values_with_comma,
                array_node.values().iter().collect_vec(),
                accessors,
                current_schema,
                schema_context,
                values_order_group,
            )
            .await
        }
    };

    let Some(mut sorted_values_with_comma) = sorted_values_with_comma else {
        return Vec::with_capacity(0);
    };

    if let Some((_, comma)) = sorted_values_with_comma.last_mut() {
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

    for (value, comma) in &sorted_values_with_comma {
        changes.extend(array_comma_trailing_comment(value, comma.as_ref()));
    }

    let new = sorted_values_with_comma
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

async fn get_sorted_values_order_all<'a>(
    values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    array_node: &'a tombi_document_tree::Array,
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    order: ArrayValuesOrder,
) -> Option<Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>> {
    let sortable_values = match SortableValues::try_new(
        values_with_comma,
        &array_node.values().iter().collect_vec(),
        accessors,
        current_schema,
        schema_context,
    )
    .await
    {
        Ok(sortable_values) => sortable_values,
        Err(reason) => {
            tracing::debug!("{reason}");
            return None;
        }
    };
    Some(sort_array_values(sortable_values, order))
}

async fn get_sorted_values_order_groups<'a>(
    mut values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    mut value_nodes: Vec<&'a tombi_document_tree::Value>,
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    values_order_group: ArrayValuesOrderGroup,
) -> Option<Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>> {
    let current_schema = current_schema?;

    match (values_order_group, current_schema.value_schema.as_ref()) {
        (
            ArrayValuesOrderGroup::OneOf(group_orders),
            ValueSchema::OneOf(OneOfSchema { schemas, .. }),
        )
        | (
            ArrayValuesOrderGroup::AnyOf(group_orders),
            ValueSchema::AnyOf(AnyOfSchema { schemas, .. }),
        ) => {
            let mut sorted_values_with_comma = Vec::with_capacity(values_with_comma.len());
            let mut schemas = schemas.write().await;

            for (group_order, schema) in group_orders.iter().zip(schemas.iter_mut()) {
                let mut group_values_with_comma = Vec::new();
                let mut group_value_nodes = Vec::new();
                let Ok(Some(current_schema)) = schema
                    .resolve(
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                    )
                    .await
                else {
                    continue;
                };

                let mut i = 0;
                while i < values_with_comma.len() {
                    // check if the value is compatible with the schema
                    if value_nodes[i]
                        .validate(&[], Some(&current_schema), schema_context)
                        .await
                        .is_ok()
                    {
                        group_values_with_comma.push(values_with_comma.remove(i));
                        group_value_nodes.push(value_nodes.remove(i));
                    } else {
                        i += 1;
                    }
                }

                // Sort group values
                if !group_values_with_comma.is_empty() {
                    match SortableValues::try_new(
                        group_values_with_comma.clone(),
                        group_value_nodes.as_slice(),
                        accessors,
                        Some(&current_schema),
                        schema_context,
                    )
                    .await
                    {
                        Ok(sortable_values) => {
                            sorted_values_with_comma
                                .append(&mut sort_array_values(sortable_values, *group_order));
                        }
                        Err(warning) => {
                            tracing::warn!("{warning}");
                            sorted_values_with_comma.append(&mut group_values_with_comma);
                        }
                    }
                }
            }

            // Append remaining values
            sorted_values_with_comma.append(&mut values_with_comma);
            Some(sorted_values_with_comma)
        }
        _ => None,
    }
}

fn sort_array_values(
    sortable_values: SortableValues,
    values_order: ArrayValuesOrder,
) -> Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)> {
    match values_order {
        ArrayValuesOrder::Ascending => sortable_values
            .sorted()
            .into_iter()
            .map(|(value, comma)| (value, Some(comma)))
            .collect_vec(),
        ArrayValuesOrder::Descending => sortable_values
            .sorted()
            .into_iter()
            .rev()
            .map(|(value, comma)| (value, Some(comma)))
            .collect_vec(),
        ArrayValuesOrder::VersionSort => sortable_values
            .sorted_version()
            .into_iter()
            .map(|(value, comma)| (value, Some(comma)))
            .collect_vec(),
    }
}

fn try_array_values_order_by_from_item_schema<'a: 'b, 'b>(
    table_node: &'a tombi_document_tree::Table,
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
) -> BoxFuture<'b, Result<ArrayValuesOrderBy, SortFailReason>> {
    async move {
        if let Some(current_schema) = current_schema {
            match current_schema.value_schema.as_ref() {
                ValueSchema::Table(TableSchema {
                    array_values_order_by: Some(array_values_order_by),
                    ..
                }) => {
                    return Ok(array_values_order_by.to_owned());
                }
                ValueSchema::AllOf(AllOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                | ValueSchema::OneOf(OneOfSchema { schemas, .. }) => {
                    for schema in schemas.write().await.iter_mut() {
                        if let Ok(Some(current_schema)) = schema
                            .resolve(
                                current_schema.schema_uri.clone(),
                                current_schema.definitions.clone(),
                                schema_context.store,
                            )
                            .await
                        {
                            if table_node
                                .validate(accessors, Some(&current_schema), schema_context)
                                .await
                                .is_ok()
                            {
                                return try_array_values_order_by_from_item_schema(
                                    table_node,
                                    accessors,
                                    Some(&current_schema),
                                    schema_context,
                                )
                                .await;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Err(SortFailReason::ArrayValuesOrderByRequired)
    }
    .boxed()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SortableType {
    Boolean,
    Integer,
    String,
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
}

enum SortableValues {
    Boolean(Vec<(bool, tombi_ast::Value, tombi_ast::Comma)>),
    Integer(Vec<(i64, tombi_ast::Value, tombi_ast::Comma)>),
    String(Vec<(String, tombi_ast::Value, tombi_ast::Comma)>),
    OffsetDateTime(Vec<(String, tombi_ast::Value, tombi_ast::Comma)>),
    LocalDateTime(Vec<(String, tombi_ast::Value, tombi_ast::Comma)>),
    LocalDate(Vec<(String, tombi_ast::Value, tombi_ast::Comma)>),
    LocalTime(Vec<(String, tombi_ast::Value, tombi_ast::Comma)>),
}

impl SortableType {
    fn try_new<'a: 'b, 'b>(
        value: &'a tombi_ast::Value,
        value_node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a SchemaContext<'a>,
    ) -> BoxFuture<'b, Result<Self, SortFailReason>> {
        async move {
            match (value, value_node) {
                (tombi_ast::Value::Boolean(_), tombi_document_tree::Value::Boolean(_)) => {
                    Ok(SortableType::Boolean)
                }
                (
                    tombi_ast::Value::IntegerBin(_)
                    | tombi_ast::Value::IntegerOct(_)
                    | tombi_ast::Value::IntegerDec(_)
                    | tombi_ast::Value::IntegerHex(_),
                    tombi_document_tree::Value::Integer(_),
                ) => Ok(SortableType::Integer),
                (
                    tombi_ast::Value::BasicString(_)
                    | tombi_ast::Value::LiteralString(_)
                    | tombi_ast::Value::MultiLineBasicString(_)
                    | tombi_ast::Value::MultiLineLiteralString(_),
                    tombi_document_tree::Value::String(_),
                ) => Ok(SortableType::String),
                (
                    tombi_ast::Value::OffsetDateTime(_),
                    tombi_document_tree::Value::OffsetDateTime(_),
                ) => Ok(SortableType::OffsetDateTime),
                (
                    tombi_ast::Value::LocalDateTime(_),
                    tombi_document_tree::Value::LocalDateTime(_),
                ) => Ok(SortableType::LocalDateTime),
                (tombi_ast::Value::LocalDate(_), tombi_document_tree::Value::LocalDate(_)) => {
                    Ok(SortableType::LocalDate)
                }
                (tombi_ast::Value::LocalTime(_), tombi_document_tree::Value::LocalTime(_)) => {
                    Ok(SortableType::LocalTime)
                }
                (
                    tombi_ast::Value::InlineTable(inline_table),
                    tombi_document_tree::Value::Table(table_node),
                ) => {
                    let array_values_order_by = try_array_values_order_by_from_item_schema(
                        table_node,
                        accessors,
                        current_schema,
                        schema_context,
                    )
                    .await?;

                    for key_value in inline_table.key_values() {
                        if let Some(keys) = key_value.keys() {
                            let mut keys_iter = keys.keys();
                            let Some(key_text) = keys_iter
                                .next()
                                .map(|key| key.to_raw_text(schema_context.toml_version))
                            else {
                                continue;
                            };
                            if key_text != array_values_order_by {
                                continue;
                            }
                            // dotted keys is not supported
                            if keys_iter.next().is_some() {
                                return Err(SortFailReason::DottedKeysInlineTableNotSupported);
                            }
                            if let (Some(value), Some(value_node)) =
                                (&key_value.value(), table_node.get(&key_text))
                            {
                                return SortableType::try_new(
                                    value,
                                    value_node,
                                    accessors,
                                    current_schema,
                                    schema_context,
                                )
                                .await;
                            } else {
                                return Err(SortFailReason::Incomplete);
                            }
                        }
                    }
                    Err(SortFailReason::ArrayValuesOrderByKeyNotFound)
                }
                (tombi_ast::Value::Float(_), tombi_document_tree::Value::Float(_))
                | (tombi_ast::Value::Array(_), tombi_document_tree::Value::Array(_)) => {
                    Err(SortFailReason::UnsupportedTypes)
                }
                _ => Err(SortFailReason::UnsupportedTypes),
            }
        }
        .boxed()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
enum SortFailReason {
    #[error("Cannot sort array values because the values are incomplete.")]
    Incomplete,

    #[error("Cannot sort array values because the values only support the following types: [Boolean, Integer, String, OffsetDateTime, LocalDateTime, LocalDate, LocalTime, InlineTable(need `x-tombi-array-values-order-by`)]")]
    UnsupportedTypes,

    #[error("Cannot sort array values because the values have different types.")]
    DifferentTypes,

    #[error("Cannot sort array tables because the `x-tombi-array-values-order-by` is required.")]
    ArrayValuesOrderByRequired,

    #[error(
        "Cannot sort array tables because the sort-key defined in `x-tombi-array-values-order-by` is not found."
    )]
    ArrayValuesOrderByKeyNotFound,

    #[error("Cannot sort array values because the values have dotted keys inline table.")]
    DottedKeysInlineTableNotSupported,
}

impl SortableValues {
    pub async fn try_new<'a>(
        values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
        value_nodes: &'a [&'a tombi_document_tree::Value],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a SchemaContext<'a>,
    ) -> Result<Self, SortFailReason> {
        let mut values_with_comma_iter = values_with_comma.iter().zip(value_nodes).enumerate();

        let sortable_type =
            if let Some((index, ((value, _), value_node))) = values_with_comma_iter.next() {
                SortableType::try_new(
                    value,
                    value_node,
                    &accessors
                        .iter()
                        .cloned()
                        .chain(std::iter::once(Accessor::Index(index)))
                        .collect_vec(),
                    current_schema,
                    schema_context,
                )
                .await?
            } else {
                unreachable!("values_with_comma is not empty");
            };

        for (index, ((value, _), value_node)) in values_with_comma_iter {
            if SortableType::try_new(
                value,
                value_node,
                &accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(Accessor::Index(index)))
                    .collect_vec(),
                current_schema,
                schema_context,
            )
            .await
                != Ok(sortable_type)
            {
                return Err(SortFailReason::DifferentTypes);
            }
        }

        let sortable_values = match sortable_type {
            SortableType::Boolean => {
                create_boolean_sortable_values(
                    values_with_comma,
                    value_nodes,
                    accessors,
                    current_schema,
                    schema_context,
                )
                .await?
            }
            SortableType::Integer => {
                create_integer_sortable_values(
                    values_with_comma,
                    value_nodes,
                    accessors,
                    current_schema,
                    schema_context,
                )
                .await?
            }
            SortableType::OffsetDateTime => {
                create_offset_date_time_sortable_values(
                    values_with_comma,
                    value_nodes,
                    accessors,
                    current_schema,
                    schema_context,
                )
                .await?
            }
            SortableType::LocalDateTime => {
                create_local_date_time_sortable_values(
                    values_with_comma,
                    value_nodes,
                    accessors,
                    current_schema,
                    schema_context,
                )
                .await?
            }
            SortableType::LocalDate => {
                create_local_date_sortable_values(
                    values_with_comma,
                    value_nodes,
                    accessors,
                    current_schema,
                    schema_context,
                )
                .await?
            }

            SortableType::LocalTime => {
                create_local_time_sortable_values(
                    values_with_comma,
                    value_nodes,
                    accessors,
                    current_schema,
                    schema_context,
                )
                .await?
            }

            SortableType::String => {
                create_string_sortable_values(
                    values_with_comma,
                    value_nodes,
                    accessors,
                    current_schema,
                    schema_context,
                )
                .await?
            }
        };

        Ok(sortable_values)
    }

    pub fn sorted(self) -> Vec<(tombi_ast::Value, tombi_ast::Comma)> {
        match self {
            Self::Boolean(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| *key);

                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
            Self::Integer(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| *key);

                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
            Self::String(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| key.clone());

                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
            Self::OffsetDateTime(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| key.clone());

                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
            Self::LocalDateTime(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| key.clone());

                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
            Self::LocalDate(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| key.clone());

                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
            Self::LocalTime(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| key.clone());

                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
        }
    }

    pub fn sorted_version(self) -> Vec<(tombi_ast::Value, tombi_ast::Comma)> {
        match self {
            Self::String(mut sortable_values) => {
                sortable_values
                    .sort_by(|(a, _, _), (b, _, _)| tombi_version_sort::version_sort(a, b));
                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
            _ => self.sorted(),
        }
    }
}
