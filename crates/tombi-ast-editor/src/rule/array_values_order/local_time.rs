use itertools::Itertools;
use tombi_ast::{AstNode, DanglingCommentGroupOr};
use tombi_schema_store::{Accessor, CurrentSchema, SchemaContext};

use crate::{
    node::make_comma,
    rule::array_values_order::{
        SortFailReason, SortableValues, try_array_values_order_by_from_item_schema,
    },
};

pub async fn create_local_time_sortable_values<'a>(
    values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    value_nodes: &'a [(usize, &'a tombi_document_tree::Value)],
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
) -> Result<SortableValues, SortFailReason> {
    let mut sortable_values = Vec::with_capacity(values_with_comma.len());
    for ((value, comma), (value_node_index, value_node)) in
        values_with_comma.into_iter().zip(value_nodes.iter())
    {
        let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());
        match (value.clone(), value_node) {
            (
                tombi_ast::Value::LocalTime(_),
                tombi_document_tree::Value::LocalTime(local_time_node),
            ) => sortable_values.push((local_time_node.to_string(), value, comma)),
            (
                tombi_ast::Value::InlineTable(inline_table),
                tombi_document_tree::Value::Table(table_node),
            ) => {
                let array_values_order_by = try_array_values_order_by_from_item_schema(
                    table_node,
                    &accessors
                        .iter()
                        .cloned()
                        .chain(std::iter::once(Accessor::Index(*value_node_index)))
                        .collect_vec(),
                    current_schema,
                    schema_context,
                )
                .await?;

                let mut found = false;
                'outer: for group in inline_table.key_value_with_comma_groups() {
                    let DanglingCommentGroupOr::ItemGroup(group) = group else {
                        continue;
                    };

                    for (key_value, comma) in group.key_values_with_comma() {
                        let Some(keys) = key_value.keys() else {
                            continue;
                        };
                        let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());

                        let mut keys_iter = keys.keys();
                        if let (Some(key), None) = (keys_iter.next(), keys_iter.next()) {
                            let key_text = key.to_raw_text(schema_context.toml_version);
                            if key_text == array_values_order_by
                                && let Some(tombi_document_tree::Value::LocalTime(local_time_node)) =
                                    table_node.get(&key_text)
                            {
                                sortable_values.push((local_time_node.to_string(), value, comma));

                                found = true;
                                break 'outer;
                            }
                        } else {
                            return Err(SortFailReason::DottedKeysInlineTableNotSupported);
                        }
                    }
                }

                if !found {
                    return Err(SortFailReason::ArrayValuesOrderByKeyNotFound);
                }
            }
            _ => return Err(SortFailReason::DifferentTypes),
        }
    }
    Ok(SortableValues::LocalTime(sortable_values))
}
