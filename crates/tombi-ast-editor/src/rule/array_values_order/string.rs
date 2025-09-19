use tombi_ast::AstNode;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_toml_version::TomlVersion;
use tombi_x_keyword::ArrayValuesOrderBy;

use crate::{
    node::make_comma,
    rule::array_values_order::{SortFailReason, SortableValues},
};

pub fn create_string_sortable_values(
    values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    array_values_order_by: Option<&ArrayValuesOrderBy>,
    toml_version: TomlVersion,
) -> Result<SortableValues, SortFailReason> {
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
                    return Err(SortFailReason::Incomplete);
                }
            }
            tombi_ast::Value::LiteralString(literal_string) => {
                if let Ok(tombi_document_tree::Value::String(string)) =
                    literal_string.try_into_document_tree(toml_version)
                {
                    sortable_values.push((string.value().to_owned(), value, comma));
                } else {
                    return Err(SortFailReason::Incomplete);
                }
            }
            tombi_ast::Value::MultiLineBasicString(multi_line_basic_string) => {
                if let Ok(tombi_document_tree::Value::String(string)) =
                    multi_line_basic_string.try_into_document_tree(toml_version)
                {
                    sortable_values.push((string.value().to_owned(), value, comma));
                } else {
                    return Err(SortFailReason::Incomplete);
                }
            }
            tombi_ast::Value::MultiLineLiteralString(multi_line_literal_string) => {
                if let Ok(tombi_document_tree::Value::String(string)) =
                    multi_line_literal_string.try_into_document_tree(toml_version)
                {
                    sortable_values.push((string.value().to_owned(), value, comma));
                } else {
                    return Err(SortFailReason::Incomplete);
                }
            }
            tombi_ast::Value::InlineTable(inline_table) => {
                let array_values_order_by =
                    array_values_order_by.ok_or(SortFailReason::ArrayValuesOrderByRequired)?;

                let mut found = false;
                for (key_value, comma) in inline_table.key_values_with_comma() {
                    let Some(keys) = key_value.keys() else {
                        continue;
                    };
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());

                    let mut keys_iter = keys.keys().into_iter();
                    if let (Some(key), None) = (keys_iter.next(), keys_iter.next()) {
                        if key.to_raw_text(toml_version) == *array_values_order_by {
                            if let Some(inline_value) = key_value.value() {
                                let document_tree_value_result = match inline_value {
                                    tombi_ast::Value::BasicString(string) => {
                                        string.try_into_document_tree(toml_version)
                                    }
                                    tombi_ast::Value::LiteralString(string) => {
                                        string.try_into_document_tree(toml_version)
                                    }
                                    tombi_ast::Value::MultiLineBasicString(string) => {
                                        string.try_into_document_tree(toml_version)
                                    }
                                    tombi_ast::Value::MultiLineLiteralString(string) => {
                                        string.try_into_document_tree(toml_version)
                                    }
                                    _ => return Err(SortFailReason::Incomplete),
                                };
                                let Ok(tombi_document_tree::Value::String(string)) =
                                    document_tree_value_result
                                else {
                                    return Err(SortFailReason::Incomplete);
                                };
                                sortable_values.push((string.value().to_owned(), value, comma));

                                found = true;
                                break;
                            }
                        }
                    } else {
                        return Err(SortFailReason::DottedKeysInlineTableNotSupported);
                    }
                }

                if !found {
                    return Err(SortFailReason::ArrayValuesOrderByKeyNotFound);
                }
            }
            _ => return Err(SortFailReason::UnsupportedTypes),
        }
    }
    Ok(SortableValues::String(sortable_values))
}
