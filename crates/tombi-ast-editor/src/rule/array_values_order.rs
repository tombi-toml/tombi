use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_schema_store::{
    AnyOfSchema, ArraySchema, CurrentSchema, OneOfSchema, SchemaContext, TableSchema, ValueSchema,
    XTombiArrayValuesOrder,
};
use tombi_syntax::SyntaxElement;
use tombi_toml_version::TomlVersion;
use tombi_validator::Validate;
use tombi_x_keyword::{ArrayValuesOrder, ArrayValuesOrderBy, ArrayValuesOrderGroup};

use crate::node::make_comma;

use super::array_comma_trailing_comment;

pub async fn array_values_order<'a>(
    mut values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    array_schema: &'a ArraySchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a SchemaContext<'a>,
) -> Vec<crate::Change> {
    if values_with_comma.is_empty() {
        return Vec::with_capacity(0);
    }

    let Some(values_order) = &array_schema.values_order else {
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

    let mut sorted_values_with_comma = match values_order {
        XTombiArrayValuesOrder::All(values_order) => {
            let array_values_order_by = if let Some(item_schema) = &array_schema.items {
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
                    get_array_values_order_by(&current_schema)
                } else {
                    return Vec::with_capacity(0);
                }
            } else {
                None
            };
            let sortable_values = match SortableValues::new(
                values_with_comma,
                array_values_order_by.as_ref(),
                schema_context.toml_version,
            ) {
                Ok(sortable_values) => sortable_values,
                Err(warning) => {
                    tracing::warn!("{warning}");
                    return Vec::with_capacity(0);
                }
            };
            sort_array_values(sortable_values, values_order)
        }
        XTombiArrayValuesOrder::Groups(values_order_group) => {
            let Some(item_schema) = &array_schema.items else {
                return Vec::with_capacity(0);
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
                return Vec::with_capacity(0);
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
                    let mut sorted_values_with_comma = Vec::new();
                    let mut schemas = schemas.write().await;

                    // group_orders と schemas を zip でループして、それぞれの schema の validate が満たされた値ごとにグループ化
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
                            // 値がスキーマに適合するかチェック
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

                        // グループ内の値をソート
                        if !group_values_with_comma.is_empty() {
                            match SortableValues::new(
                                group_values_with_comma.clone(),
                                get_array_values_order_by(&current_schema).as_ref(),
                                schema_context.toml_version,
                            ) {
                                Ok(sortable_values) => {
                                    sorted_values_with_comma.append(&mut sort_array_values(
                                        sortable_values,
                                        group_order,
                                    ));
                                }
                                Err(warning) => {
                                    tracing::warn!("{warning}");
                                    // ソートに失敗した場合は元の順序で追加
                                    sorted_values_with_comma.append(&mut group_values_with_comma);
                                }
                            }
                        }
                    }

                    // どのスキーマにも適合しなかった残りの値を最後に追加
                    sorted_values_with_comma.append(&mut values_with_comma);
                    sorted_values_with_comma
                }
                _ => return Vec::with_capacity(0),
            }
        }
    };

    if let Some((_, comma)) = sorted_values_with_comma.last_mut() {
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

    for (value, comma) in &sorted_values_with_comma {
        changes.extend(array_comma_trailing_comment(
            value,
            comma.as_ref(),
            schema_context,
        ));
    }

    let new = sorted_values_with_comma
        .iter()
        .flat_map(|(value, comma)| {
            if let Some(comma) = comma {
                vec![
                    SyntaxElement::Node(value.syntax().clone()),
                    SyntaxElement::Node(comma.syntax().clone()),
                ]
            } else {
                vec![SyntaxElement::Node(value.syntax().clone())]
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

fn sort_array_values(
    sortable_values: SortableValues,
    values_order: &ArrayValuesOrder,
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

fn get_array_values_order_by<'a>(
    current_schema: &'a CurrentSchema<'a>,
) -> Option<ArrayValuesOrderBy> {
    if let ValueSchema::Table(TableSchema {
        array_values_order_by,
        ..
    }) = current_schema.value_schema.as_ref()
    {
        array_values_order_by.to_owned()
    } else {
        None
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
    ) -> Result<Self, Warning> {
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
                            if keys_iter.next().is_some() {
                                continue;
                            }
                            if let Some(value) = &key_value.value() {
                                return SortableType::try_new(value, None, toml_version);
                            }
                        }
                    }
                    Err(Warning::ArrayValuesOrderByKeyNotFound)
                } else {
                    Err(Warning::ArrayValuesOrderByRequired)
                }
            }
            tombi_ast::Value::Float(_) | tombi_ast::Value::Array(_) => {
                Err(Warning::UnsupportedTypes)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
enum Warning {
    #[error("Cannot sort array values because the values are empty.")]
    Empty,

    #[error("Cannot sort array values because the values are incomplete.")]
    Incomplete,

    #[error("Cannot sort array values because the values only support the following types: [Boolean, Integer, String, OffsetDateTime, LocalDateTime, LocalDate, LocalTime]")]
    UnsupportedTypes,

    #[error("Cannot sort array values because the values have different types.")]
    DifferentTypes,

    #[error("Cannot sort array tables because the `x-tombi-array-values-order-by` is required.")]
    ArrayValuesOrderByRequired,

    #[error(
        "Cannot sort array tables because the sort-key defined in `x-tombi-array-values-order-by` is not found."
    )]
    ArrayValuesOrderByKeyNotFound,
}

impl SortableValues {
    pub fn new(
        values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
        array_values_order_by: Option<&ArrayValuesOrderBy>,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> Result<Self, Warning> {
        if values_with_comma.is_empty() {
            return Err(Warning::UnsupportedTypes);
        }
        let mut values_with_comma_iter = values_with_comma.iter();

        let Some(sortable_type) = values_with_comma_iter
            .next()
            .map(|(value, _)| SortableType::try_new(value, array_values_order_by, toml_version))
            .transpose()?
        else {
            return Err(Warning::Empty);
        };

        if values_with_comma_iter.any(|(value, _)| {
            SortableType::try_new(value, array_values_order_by, toml_version) != Ok(sortable_type)
        }) {
            return Err(Warning::DifferentTypes);
        }

        let sortable_values = match sortable_type {
            SortableType::Boolean => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
                    if let tombi_ast::Value::Boolean(_) = value {
                        match value.syntax().to_string().as_ref() {
                            "true" => sortable_values.push((true, value, comma)),
                            "false" => sortable_values.push((false, value, comma)),
                            _ => return Err(Warning::Incomplete),
                        }
                    } else {
                        return Err(Warning::DifferentTypes);
                    }
                }
                SortableValues::Boolean(sortable_values)
            }
            SortableType::Integer => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
                    match value.clone() {
                        tombi_ast::Value::IntegerBin(integer_bin) => {
                            if let Ok(tombi_document_tree::Value::Integer(integer)) =
                                integer_bin.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((integer.value(), value, comma));
                            } else {
                                return Err(Warning::Incomplete);
                            }
                        }
                        tombi_ast::Value::IntegerOct(integer_oct) => {
                            if let Ok(tombi_document_tree::Value::Integer(integer)) =
                                integer_oct.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((integer.value(), value, comma));
                            } else {
                                return Err(Warning::Incomplete);
                            }
                        }
                        tombi_ast::Value::IntegerDec(integer_dec) => {
                            if let Ok(tombi_document_tree::Value::Integer(integer)) =
                                integer_dec.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((integer.value(), value, comma));
                            } else {
                                return Err(Warning::Incomplete);
                            }
                        }
                        tombi_ast::Value::IntegerHex(integer_hex) => {
                            if let Ok(tombi_document_tree::Value::Integer(integer)) =
                                integer_hex.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((integer.value(), value, comma));
                            } else {
                                return Err(Warning::Incomplete);
                            }
                        }
                        _ => return Err(Warning::DifferentTypes),
                    }
                }
                SortableValues::Integer(sortable_values)
            }
            SortableType::String => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
                    match value.clone() {
                        tombi_ast::Value::BasicString(basic_string) => {
                            if let Ok(tombi_document_tree::Value::String(string)) =
                                basic_string.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((string.value().to_owned(), value, comma));
                            } else {
                                return Err(Warning::Incomplete);
                            }
                        }
                        tombi_ast::Value::LiteralString(literal_string) => {
                            if let Ok(tombi_document_tree::Value::String(string)) =
                                literal_string.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((string.value().to_owned(), value, comma));
                            } else {
                                return Err(Warning::Incomplete);
                            }
                        }
                        tombi_ast::Value::MultiLineBasicString(multi_line_basic_string) => {
                            if let Ok(tombi_document_tree::Value::String(string)) =
                                multi_line_basic_string.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((string.value().to_owned(), value, comma));
                            } else {
                                return Err(Warning::Incomplete);
                            }
                        }
                        tombi_ast::Value::MultiLineLiteralString(multi_line_literal_string) => {
                            if let Ok(tombi_document_tree::Value::String(string)) =
                                multi_line_literal_string.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((string.value().to_owned(), value, comma));
                            } else {
                                return Err(Warning::Incomplete);
                            }
                        }
                        tombi_ast::Value::InlineTable(inline_table) => {
                            let array_values_order_by =
                                array_values_order_by.ok_or(Warning::ArrayValuesOrderByRequired)?;

                            let mut found = false;
                            for key_value in inline_table.key_values() {
                                let keys = key_value
                                    .keys()
                                    .ok_or(Warning::ArrayValuesOrderByKeyNotFound)?;
                                let mut keys_iter = keys.keys().into_iter();

                                if let (Some(key), None) = (keys_iter.next(), keys_iter.next()) {
                                    if key.to_raw_text(toml_version) == *array_values_order_by {
                                        if let Some(key_value) = &key_value.value() {
                                            if let Ok(tombi_document_tree::Value::String(string)) =
                                                key_value
                                                    .clone()
                                                    .try_into_document_tree(toml_version)
                                            {
                                                sortable_values.push((
                                                    string.value().to_owned(),
                                                    value.clone(),
                                                    comma,
                                                ));
                                                found = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }

                            if !found {
                                return Err(Warning::ArrayValuesOrderByKeyNotFound);
                            }
                        }
                        _ => return Err(Warning::UnsupportedTypes),
                    }
                }
                SortableValues::String(sortable_values)
            }
            SortableType::OffsetDateTime => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
                    if let tombi_ast::Value::OffsetDateTime(_) = value {
                        sortable_values.push((value.syntax().to_string(), value, comma));
                    } else {
                        return Err(Warning::DifferentTypes);
                    }
                }
                SortableValues::OffsetDateTime(sortable_values)
            }
            SortableType::LocalDateTime => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
                    if let tombi_ast::Value::LocalDateTime(_) = value {
                        sortable_values.push((value.syntax().to_string(), value, comma));
                    } else {
                        return Err(Warning::DifferentTypes);
                    }
                }
                SortableValues::LocalDateTime(sortable_values)
            }
            SortableType::LocalDate => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
                    if let tombi_ast::Value::LocalDate(_) = value {
                        sortable_values.push((value.syntax().to_string(), value, comma));
                    } else {
                        return Err(Warning::DifferentTypes);
                    }
                }
                SortableValues::LocalDate(sortable_values)
            }
            SortableType::LocalTime => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
                    if let tombi_ast::Value::LocalTime(_) = value {
                        sortable_values.push((value.syntax().to_string(), value, comma));
                    } else {
                        return Err(Warning::DifferentTypes);
                    }
                }
                SortableValues::LocalTime(sortable_values)
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
