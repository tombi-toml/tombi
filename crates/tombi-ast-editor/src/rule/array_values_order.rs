use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_schema_store::{ArraySchema, SchemaContext};
use tombi_syntax::SyntaxElement;
use tombi_x_keyword::ArrayValuesOrder;

use crate::node::make_comma;

use super::array_comma_trailing_comment;

pub async fn array_values_order<'a>(
    values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    array_schema: &'a ArraySchema,
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

    let sortable_values =
        match SortableValues::new(values_with_comma.clone(), schema_context.toml_version) {
            Ok(sortable_values) => sortable_values,
            Err(warning) => {
                tracing::debug!("{warning}");
                return Vec::with_capacity(0);
            }
        };

    let mut sorted_values_with_comma = match values_order {
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
}

impl SortableValues {
    pub fn new(
        values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> Result<Self, Warning> {
        if values_with_comma.is_empty() {
            return Err(Warning::UnsupportedTypes);
        }

        let sortable_type = match values_with_comma.first().unwrap().0 {
            tombi_ast::Value::Boolean(_) => SortableType::Boolean,
            tombi_ast::Value::IntegerBin(_)
            | tombi_ast::Value::IntegerOct(_)
            | tombi_ast::Value::IntegerDec(_)
            | tombi_ast::Value::IntegerHex(_) => SortableType::Integer,
            tombi_ast::Value::BasicString(_)
            | tombi_ast::Value::LiteralString(_)
            | tombi_ast::Value::MultiLineBasicString(_)
            | tombi_ast::Value::MultiLineLiteralString(_) => SortableType::String,
            tombi_ast::Value::OffsetDateTime(_) => SortableType::OffsetDateTime,
            tombi_ast::Value::LocalDateTime(_) => SortableType::LocalDateTime,
            tombi_ast::Value::LocalDate(_) => SortableType::LocalDate,
            tombi_ast::Value::LocalTime(_) => SortableType::LocalTime,
            _ => return Err(Warning::Empty),
        };

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
