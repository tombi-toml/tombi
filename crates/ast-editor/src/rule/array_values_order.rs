use ast::AstNode;
use document_tree::TryIntoDocumentTree;
use itertools::Itertools;
use schema_store::{SchemaContext, ValueSchema};
use syntax::SyntaxElement;
use x_tombi::ArrayValuesOrder;

pub async fn array_values_order<'a>(
    values_with_comma: Vec<(ast::Value, Option<ast::Comma>)>,
    value_schema: &'a ValueSchema,
    schema_context: &'a SchemaContext<'a>,
) -> Vec<crate::Change> {
    if values_with_comma.is_empty() {
        return Vec::with_capacity(0);
    }

    let is_last_comma = values_with_comma
        .last()
        .map(|(_, comma)| comma.is_some())
        .unwrap_or(false);

    let ValueSchema::Array(array_schema) = value_schema else {
        return Vec::with_capacity(0);
    };

    let Some(values_order) = &array_schema.values_order else {
        return Vec::with_capacity(0);
    };

    let old = std::ops::RangeInclusive::new(
        SyntaxElement::Node(values_with_comma.first().unwrap().0.syntax().clone()),
        SyntaxElement::Node(values_with_comma.last().unwrap().0.syntax().clone()),
    );

    let sortable_values = match SortableValues::new(values_with_comma, schema_context.toml_version)
    {
        Ok(sortable_values) => sortable_values,
        Err(err) => {
            tracing::error!("{err}");
            return Vec::with_capacity(0);
        }
    };

    let new = match values_order {
        ArrayValuesOrder::Ascending => sortable_values
            .sorted()
            .into_iter()
            .map(|(value, comma)| {
                let mut elements = vec![SyntaxElement::Node(value.syntax().clone())];
                if let Some(comma) = comma {
                    elements.push(SyntaxElement::Node(comma.syntax().clone()));
                }
                elements
            })
            .flatten()
            .collect_vec(),
        ArrayValuesOrder::Descending => sortable_values
            .sorted()
            .into_iter()
            .rev()
            .map(|(value, comma)| {
                let mut elements = vec![SyntaxElement::Node(value.syntax().clone())];
                if let Some(comma) = comma {
                    elements.push(SyntaxElement::Node(comma.syntax().clone()));
                }
                elements
            })
            .flatten()
            .collect_vec(),
    };

    let mut changes = Vec::with_capacity(2);

    if !is_last_comma {
        if let Some(syntax::SyntaxElement::Node(node)) = new.last() {
            if let Some(comma) = ast::Comma::cast(node.clone().into()) {
                if comma.tailing_comment().is_none()
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
    Boolean(Vec<(bool, ast::Value, Option<ast::Comma>)>),
    Integer(Vec<(i64, ast::Value, Option<ast::Comma>)>),
    String(Vec<(String, ast::Value, Option<ast::Comma>)>),
    OffsetDateTime(Vec<(String, ast::Value, Option<ast::Comma>)>),
    LocalDateTime(Vec<(String, ast::Value, Option<ast::Comma>)>),
    LocalDate(Vec<(String, ast::Value, Option<ast::Comma>)>),
    LocalTime(Vec<(String, ast::Value, Option<ast::Comma>)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
enum Error {
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
        values_with_comma: Vec<(ast::Value, Option<ast::Comma>)>,
        toml_version: toml_version::TomlVersion,
    ) -> Result<Self, Error> {
        if values_with_comma.is_empty() {
            return Err(Error::UnsupportedTypes);
        }

        let sortable_type = match values_with_comma.first().unwrap().0 {
            ast::Value::Boolean(_) => SortableType::Boolean,
            ast::Value::IntegerBin(_)
            | ast::Value::IntegerOct(_)
            | ast::Value::IntegerDec(_)
            | ast::Value::IntegerHex(_) => SortableType::Integer,
            ast::Value::BasicString(_)
            | ast::Value::LiteralString(_)
            | ast::Value::MultiLineBasicString(_)
            | ast::Value::MultiLineLiteralString(_) => SortableType::String,
            ast::Value::OffsetDateTime(_) => SortableType::OffsetDateTime,
            ast::Value::LocalDateTime(_) => SortableType::LocalDateTime,
            ast::Value::LocalDate(_) => SortableType::LocalDate,
            ast::Value::LocalTime(_) => SortableType::LocalTime,
            _ => return Err(Error::Empty),
        };

        let sortable_values = match sortable_type {
            SortableType::Boolean => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    if let ast::Value::Boolean(_) = value {
                        match value.syntax().to_string().as_ref() {
                            "true" => sortable_values.push((true, value, comma)),
                            "false" => sortable_values.push((false, value, comma)),
                            _ => return Err(Error::Incomplete),
                        }
                    } else {
                        return Err(Error::DifferentTypes);
                    }
                }
                SortableValues::Boolean(sortable_values)
            }
            SortableType::Integer => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    match value.clone() {
                        ast::Value::IntegerBin(integer_bin) => {
                            if let Ok(document_tree::Value::Integer(integer)) =
                                integer_bin.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((integer.value(), value, comma));
                            } else {
                                return Err(Error::Incomplete);
                            }
                        }
                        ast::Value::IntegerOct(integer_oct) => {
                            if let Ok(document_tree::Value::Integer(integer)) =
                                integer_oct.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((integer.value(), value, comma));
                            } else {
                                return Err(Error::Incomplete);
                            }
                        }
                        ast::Value::IntegerDec(integer_dec) => {
                            if let Ok(document_tree::Value::Integer(integer)) =
                                integer_dec.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((integer.value(), value, comma));
                            } else {
                                return Err(Error::Incomplete);
                            }
                        }
                        ast::Value::IntegerHex(integer_hex) => {
                            if let Ok(document_tree::Value::Integer(integer)) =
                                integer_hex.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((integer.value(), value, comma));
                            } else {
                                return Err(Error::Incomplete);
                            }
                        }
                        _ => return Err(Error::DifferentTypes),
                    }
                }
                SortableValues::Integer(sortable_values)
            }
            SortableType::String => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    match value.clone() {
                        ast::Value::BasicString(basic_string) => {
                            if let Ok(document_tree::Value::String(string)) =
                                basic_string.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((string.value().to_owned(), value, comma));
                            } else {
                                return Err(Error::Incomplete);
                            }
                        }
                        ast::Value::LiteralString(literal_string) => {
                            if let Ok(document_tree::Value::String(string)) =
                                literal_string.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((string.value().to_owned(), value, comma));
                            } else {
                                return Err(Error::Incomplete);
                            }
                        }
                        ast::Value::MultiLineBasicString(multi_line_basic_string) => {
                            if let Ok(document_tree::Value::String(string)) =
                                multi_line_basic_string.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((string.value().to_owned(), value, comma));
                            } else {
                                return Err(Error::Incomplete);
                            }
                        }
                        ast::Value::MultiLineLiteralString(multi_line_literal_string) => {
                            if let Ok(document_tree::Value::String(string)) =
                                multi_line_literal_string.try_into_document_tree(toml_version)
                            {
                                sortable_values.push((string.value().to_owned(), value, comma));
                            } else {
                                return Err(Error::Incomplete);
                            }
                        }
                        _ => return Err(Error::UnsupportedTypes),
                    }
                }
                SortableValues::String(sortable_values)
            }
            SortableType::OffsetDateTime => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    if let ast::Value::OffsetDateTime(_) = value {
                        sortable_values.push((value.syntax().to_string(), value, comma));
                    } else {
                        return Err(Error::DifferentTypes);
                    }
                }
                SortableValues::OffsetDateTime(sortable_values)
            }
            SortableType::LocalDateTime => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    if let ast::Value::LocalDateTime(_) = value {
                        sortable_values.push((value.syntax().to_string(), value, comma));
                    } else {
                        return Err(Error::DifferentTypes);
                    }
                }
                SortableValues::LocalDateTime(sortable_values)
            }
            SortableType::LocalDate => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    if let ast::Value::LocalDate(_) = value {
                        sortable_values.push((value.syntax().to_string(), value, comma));
                    } else {
                        return Err(Error::DifferentTypes);
                    }
                }
                SortableValues::LocalDate(sortable_values)
            }
            SortableType::LocalTime => {
                let mut sortable_values = Vec::with_capacity(values_with_comma.len());
                for (value, comma) in values_with_comma {
                    if let ast::Value::LocalTime(_) = value {
                        sortable_values.push((value.syntax().to_string(), value, comma));
                    } else {
                        return Err(Error::DifferentTypes);
                    }
                }
                SortableValues::LocalTime(sortable_values)
            }
        };

        Ok(sortable_values)
    }

    pub fn sorted(self) -> Vec<(ast::Value, Option<ast::Comma>)> {
        match self {
            Self::Boolean(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| key.clone());

                sortable_values
                    .into_iter()
                    .map(|(_, value, comma)| (value, comma))
                    .collect_vec()
            }
            Self::Integer(mut sortable_values) => {
                sortable_values.sort_by_key(|(key, _, _)| key.clone());

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
}
