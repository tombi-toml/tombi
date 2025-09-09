use tombi_json::StringNode;
use tombi_x_keyword::{TableKeysOrder, TableKeysOrderGroup, X_TOMBI_TABLE_KEYS_ORDER};

#[derive(Debug, Clone)]
pub enum TableKeysOrderSpec {
    All(TableKeysOrder),
    Groups(Vec<GroupTableKeysOrder>),
}

#[derive(Debug, Clone)]
pub struct GroupTableKeysOrder {
    pub target: TableKeysOrderGroup,
    pub order: TableKeysOrder,
}

impl TableKeysOrderSpec {
    pub fn new(value_node: &tombi_json::ValueNode) -> Option<Self> {
        match value_node {
            tombi_json::ValueNode::String(StringNode { value: order, .. }) => {
                match TableKeysOrder::try_from(order.as_str()) {
                    Ok(val) => Some(TableKeysOrderSpec::All(val)),
                    Err(_) => {
                        tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER}: {order}");
                        None
                    }
                }
            }
            tombi_json::ValueNode::Object(object_node) => {
                let mut sort_orders = vec![];
                for (group_name, order) in &object_node.properties {
                    let Ok(target) = TableKeysOrderGroup::try_from(group_name.value.as_str())
                    else {
                        tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER} group: {group_name}");
                        return None;
                    };

                    let Some(Ok(order)) = order.as_str().map(TableKeysOrder::try_from) else {
                        tracing::warn!(
                            "Invalid {X_TOMBI_TABLE_KEYS_ORDER} {group_name} group: {order}"
                        );
                        return None;
                    };

                    if order == TableKeysOrder::Schema && target != TableKeysOrderGroup::Keys {
                        tracing::warn!(
                            "Invalid {X_TOMBI_TABLE_KEYS_ORDER} {group_name} group: {order}"
                        );
                        return None;
                    }

                    sort_orders.push(GroupTableKeysOrder { target, order });
                }
                Some(Self::Groups(sort_orders))
            }
            order => {
                tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER}: {}", order.to_string());
                None
            }
        }
    }
}
