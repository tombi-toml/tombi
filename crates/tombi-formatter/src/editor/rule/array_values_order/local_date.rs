use itertools::Itertools;
use tombi_ast_syntax::DanglingCommentGroupOr;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaContext};

use crate::editor::rule::array_values_order::{
    SortFailReason, SortableValues, try_array_values_order_by_from_item_schema,
};

pub(super) async fn create_local_date_sortable_values<'a>(
    values_with_comma: Vec<(tombi_ast_syntax::Value, Option<tombi_ast_syntax::Comma>)>,
    value_nodes: &'a [(usize, &'a tombi_document_tree_syntax::Value)],
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
) -> Result<SortableValues, SortFailReason> {
    let mut sortable_values = Vec::with_capacity(values_with_comma.len());
    for ((value, comma), (value_node_index, value_node)) in
        values_with_comma.into_iter().zip(value_nodes.iter())
    {
        match (value.clone(), value_node) {
            (
                tombi_ast_syntax::Value::LocalDate(_),
                tombi_document_tree_syntax::Value::LocalDate(local_date_node),
            ) => sortable_values.push((local_date_node.to_string(), value, comma)),
            (
                tombi_ast_syntax::Value::InlineTable(inline_table),
                tombi_document_tree_syntax::Value::Table(table_node),
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

                    for key_value in group.key_values() {
                        let Some(keys) = key_value.keys() else {
                            continue;
                        };

                        let mut keys_iter = keys.keys();
                        if let (Some(key), None) = (keys_iter.next(), keys_iter.next()) {
                            let key_text = key.content_lossy(schema_context.toml_version);
                            if key_text == array_values_order_by
                                && let Some(tombi_document_tree_syntax::Value::LocalDate(
                                    local_date_node,
                                )) = table_node.get(&key_text)
                            {
                                sortable_values.push((local_date_node.to_string(), value, comma));

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
    Ok(SortableValues::LocalDate(sortable_values))
}
