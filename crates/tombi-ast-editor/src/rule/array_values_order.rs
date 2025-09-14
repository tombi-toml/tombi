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
    ArrayCommonLintRules, ArrayFormatRules, TombiValueDirectiveContent,
};
use tombi_document_tree::TryIntoDocumentTree;
use tombi_schema_store::{
    AnyOfSchema, CurrentSchema, OneOfSchema, SchemaContext, TableSchema, ValueSchema,
    XTombiArrayValuesOrder,
};
use tombi_syntax::SyntaxElement;
use tombi_toml_version::TomlVersion;
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
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    comment_directive: Option<TombiValueDirectiveContent<ArrayFormatRules, ArrayCommonLintRules>>,
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
        None => {
            if let Some(ValueSchema::Array(array_schema)) =
                current_schema.map(|current_schema| current_schema.value_schema.as_ref())
            {
                array_schema.values_order.clone()
            } else {
                None
            }
        }
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
            sort_array_values_order_all(
                values_with_comma,
                current_schema,
                schema_context,
                values_order,
            )
            .await
        }
        XTombiArrayValuesOrder::Groups(values_order_group) => {
            sort_array_values_order_groups(
                values_with_comma,
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

async fn sort_array_values_order_all<'a>(
    values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    order: ArrayValuesOrder,
) -> Option<Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>> {
    let array_values_order_by = get_array_values_order_by(current_schema, schema_context).await;
    let sortable_values = match SortableValues::try_new(
        values_with_comma,
        array_values_order_by.as_ref(),
        schema_context.toml_version,
    ) {
        Ok(sortable_values) => sortable_values,
        Err(reason) => {
            tracing::debug!("{reason}");
            return None;
        }
    };
    Some(sort_array_values(sortable_values, order))
}

async fn sort_array_values_order_groups<'a>(
    mut values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    values_order_group: ArrayValuesOrderGroup,
) -> Option<Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>> {
    let Some(current_schema) = current_schema else {
        return None;
    };
    let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref() else {
        return None;
    };
    let Some(item_schema) = &array_schema.items else {
        return None;
    };
    let mut item_schema = item_schema.write().await;
    let Some(current_schema) = item_schema
        .resolve(
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
        )
        .await
        .ok()
        .flatten()
    else {
        return None;
    };

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
                    let (value, _) = &values_with_comma[i];
                    // check if the value is compatible with the schema
                    if let Ok(document_tree_value) = value
                        .clone()
                        .try_into_document_tree(schema_context.toml_version)
                    {
                        if document_tree_value
                            .validate(&[], Some(&current_schema), schema_context)
                            .await
                            .is_ok()
                        {
                            group_values_with_comma.push(values_with_comma.remove(i));
                        } else {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }

                // Sort group values
                if !group_values_with_comma.is_empty() {
                    match SortableValues::try_new(
                        group_values_with_comma.clone(),
                        get_array_values_order_by_from_item_schema(&current_schema).as_ref(),
                        schema_context.toml_version,
                    ) {
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

async fn get_array_values_order_by<'a>(
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
) -> Option<ArrayValuesOrderBy> {
    if let Some(current_schema) = current_schema {
        match current_schema.value_schema.as_ref() {
            ValueSchema::Array(array_schema) => {
                if let Some(item_schema) = &array_schema.items {
                    if let Some(current_schema) = item_schema
                        .write()
                        .await
                        .resolve(
                            current_schema.schema_uri.clone(),
                            current_schema.definitions.clone(),
                            schema_context.store,
                        )
                        .await
                        .ok()
                        .flatten()
                    {
                        return get_array_values_order_by_from_item_schema(&current_schema);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn get_array_values_order_by_from_item_schema<'a>(
    item_schema: &'a CurrentSchema<'a>,
) -> Option<ArrayValuesOrderBy> {
    match item_schema.value_schema.as_ref() {
        ValueSchema::Table(TableSchema {
            array_values_order_by,
            ..
        }) => array_values_order_by.to_owned(),
        _ => None,
    }
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
    fn try_new(
        value: &tombi_ast::Value,
        array_values_order_by: Option<&ArrayValuesOrderBy>,
        toml_version: TomlVersion,
    ) -> Result<Self, SortFailReason> {
        match value {
            tombi_ast::Value::Boolean(_) => Ok(SortableType::Boolean),
            tombi_ast::Value::IntegerBin(_)
            | tombi_ast::Value::IntegerOct(_)
            | tombi_ast::Value::IntegerDec(_)
            | tombi_ast::Value::IntegerHex(_) => Ok(SortableType::Integer),
            tombi_ast::Value::BasicString(_)
            | tombi_ast::Value::LiteralString(_)
            | tombi_ast::Value::MultiLineBasicString(_)
            | tombi_ast::Value::MultiLineLiteralString(_) => Ok(SortableType::String),
            tombi_ast::Value::OffsetDateTime(_) => Ok(SortableType::OffsetDateTime),
            tombi_ast::Value::LocalDateTime(_) => Ok(SortableType::LocalDateTime),
            tombi_ast::Value::LocalDate(_) => Ok(SortableType::LocalDate),
            tombi_ast::Value::LocalTime(_) => Ok(SortableType::LocalTime),
            tombi_ast::Value::InlineTable(inline_table) => {
                if let Some(array_values_order_by) = array_values_order_by {
                    for key_value in inline_table.key_values() {
                        if let Some(keys) = key_value.keys() {
                            let mut keys_iter = keys.keys().into_iter();
                            if let Some(key) = keys_iter.next() {
                                if key.to_raw_text(toml_version) != *array_values_order_by {
                                    continue;
                                }
                            }
                            // dotted keys is not supported
                            if keys_iter.next().is_some() {
                                return Err(SortFailReason::DottedKeysInlineTableNotSupported);
                            }
                            if let Some(value) = &key_value.value() {
                                return SortableType::try_new(value, None, toml_version);
                            } else {
                                return Err(SortFailReason::Incomplete);
                            }
                        }
                    }
                    Err(SortFailReason::ArrayValuesOrderByKeyNotFound)
                } else {
                    Err(SortFailReason::ArrayValuesOrderByRequired)
                }
            }
            tombi_ast::Value::Float(_) | tombi_ast::Value::Array(_) => {
                Err(SortFailReason::UnsupportedTypes)
            }
        }
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
    pub fn try_new(
        values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
        array_values_order_by: Option<&ArrayValuesOrderBy>,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> Result<Self, SortFailReason> {
        let mut values_with_comma_iter = values_with_comma.iter();

        let Some(sortable_type) = values_with_comma_iter
            .next()
            .map(|(value, _)| SortableType::try_new(value, array_values_order_by, toml_version))
            .transpose()?
        else {
            unreachable!("values_with_comma is not empty");
        };

        if values_with_comma_iter.any(|(value, _)| {
            SortableType::try_new(value, array_values_order_by, toml_version) != Ok(sortable_type)
        }) {
            return Err(SortFailReason::DifferentTypes);
        }

        let sortable_values = match sortable_type {
            SortableType::Boolean => create_boolean_sortable_values(
                values_with_comma,
                array_values_order_by,
                toml_version,
            )?,
            SortableType::Integer => create_integer_sortable_values(
                values_with_comma,
                array_values_order_by,
                toml_version,
            )?,
            SortableType::OffsetDateTime => create_offset_date_time_sortable_values(
                values_with_comma,
                array_values_order_by,
                toml_version,
            )?,
            SortableType::LocalDateTime => create_local_date_time_sortable_values(
                values_with_comma,
                array_values_order_by,
                toml_version,
            )?,
            SortableType::LocalDate => create_local_date_sortable_values(
                values_with_comma,
                array_values_order_by,
                toml_version,
            )?,
            SortableType::LocalTime => create_local_time_sortable_values(
                values_with_comma,
                array_values_order_by,
                toml_version,
            )?,
            SortableType::String => create_string_sortable_values(
                values_with_comma,
                array_values_order_by,
                toml_version,
            )?,
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
