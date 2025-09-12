use tombi_ast::AstNode;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_toml_version::TomlVersion;
use tombi_x_keyword::ArrayValuesOrderBy;

use crate::{
    node::make_comma,
    rule::array_values_order::{SortFailReason, SortableValues},
};

pub fn create_offset_date_time_sortable_values(
    values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    array_values_order_by: Option<&ArrayValuesOrderBy>,
    toml_version: TomlVersion,
) -> Result<SortableValues, SortFailReason> {
    let mut sortable_values = Vec::with_capacity(values_with_comma.len());
    for (value, comma) in values_with_comma {
        let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
        match value.clone() {
            tombi_ast::Value::OffsetDateTime(_) => {
                sortable_values.push((value.syntax().to_string(), value, comma))
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
                                    tombi_ast::Value::OffsetDateTime(offset_date_time) => {
                                        offset_date_time.try_into_document_tree(toml_version)
                                    }
                                    _ => return Err(SortFailReason::Incomplete),
                                };
                                let Ok(tombi_document_tree::Value::OffsetDateTime(
                                    offset_date_time,
                                )) = document_tree_value_result
                                else {
                                    return Err(SortFailReason::Incomplete);
                                };
                                sortable_values.push((offset_date_time.to_string(), value, comma));

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
            _ => return Err(SortFailReason::DifferentTypes),
        }
    }
    Ok(SortableValues::OffsetDateTime(sortable_values))
}
