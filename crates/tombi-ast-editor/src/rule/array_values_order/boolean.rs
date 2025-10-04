use tombi_ast::AstNode;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaContext};

use crate::{
    node::make_comma,
    rule::array_values_order::{
        try_array_values_order_by_from_item_schema, SortFailReason, SortableValues,
    },
};

pub async fn create_boolean_sortable_values<'a>(
    values_with_comma: Vec<(tombi_ast::Value, Option<tombi_ast::Comma>)>,
    value_nodes: &'a [&'a tombi_document_tree::Value],
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
) -> Result<SortableValues, SortFailReason> {
    let mut sortable_values = Vec::with_capacity(values_with_comma.len());
    for ((value, comma), value_node) in values_with_comma.into_iter().zip(value_nodes) {
        let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());

        match (value.clone(), value_node) {
            (tombi_ast::Value::Boolean(_), tombi_document_tree::Value::Boolean(boolean_node)) => {
                match boolean_node.value() {
                    true => sortable_values.push((true, value, comma)),
                    false => sortable_values.push((false, value, comma)),
                }
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

                let mut found = false;
                for (key_value, comma) in inline_table.key_values_with_comma() {
                    let Some(keys) = key_value.keys() else {
                        continue;
                    };
                    let comma = comma.unwrap_or(tombi_ast::Comma::cast(make_comma()).unwrap());

                    let mut keys_iter = keys.keys();
                    if let (Some(key), None) = (keys_iter.next(), keys_iter.next()) {
                        let key_text = key.to_raw_text(schema_context.toml_version);
                        if key_text == array_values_order_by {
                            if let Some(tombi_document_tree::Value::Boolean(boolean_node)) =
                                table_node.get(&key_text)
                            {
                                sortable_values.push((boolean_node.value(), value, comma));

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
    Ok(SortableValues::Boolean(sortable_values))
}
