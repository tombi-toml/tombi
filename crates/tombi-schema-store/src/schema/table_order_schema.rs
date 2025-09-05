use tombi_x_keyword::{TableGroup, TableKeysOrder, X_TOMBI_TABLE_KEYS_ORDER};

#[derive(Debug, Clone)]
pub enum TableOrderSchema {
    All(TableKeysOrder),
    Groups(Vec<GroupTableKeysOrder>),
}

#[derive(Debug, Clone)]
pub struct GroupTableKeysOrder {
    pub target: TableGroup,
    pub order: TableKeysOrder,
}

impl TableOrderSchema {
    pub fn new(object: &tombi_json::ObjectNode) -> Option<Self> {
        let mut sort_orders = vec![];
        for (group_name, order) in &object.properties {
            let Ok(target) = TableGroup::try_from(group_name.value.as_str()) else {
                tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER} group: {group_name}");
                return None;
            };

            let Some(Ok(order)) = order.as_str().map(TableKeysOrder::try_from) else {
                tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER}.{group_name}: {order}");
                return None;
            };

            if order == TableKeysOrder::Schema && target == TableGroup::AdditionalProperties {
                tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER}.{group_name}: {order}");
                return None;
            }

            sort_orders.push(GroupTableKeysOrder { target, order });
        }
        Some(Self::Groups(sort_orders))
    }
}
