use tombi_x_keyword::{TableGroup, TableKeysOrder, X_TOMBI_TABLE_KEYS_ORDER};

#[derive(Debug, Clone)]
pub struct TableOrderSchema {
    pub orders: Vec<TableGroupOrder>,
}

#[derive(Debug, Clone)]
pub struct TableGroupOrder {
    pub target: TableGroup,
    pub order: TableKeysOrder,
}

impl TableOrderSchema {
    pub fn new(object: &tombi_json::ObjectNode) -> Option<Self> {
        let mut sort_orders = vec![];
        for (group_name, order) in &object.properties {
            let Ok(target) = TableGroup::try_from(group_name.value.as_str()) else {
                tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER}: {group_name}");
                return None;
            };

            let Some(Ok(order)) = order.as_str().map(TableKeysOrder::try_from) else {
                tracing::warn!("Invalid {X_TOMBI_TABLE_KEYS_ORDER}.{group_name}: {order}");
                return None;
            };
            sort_orders.push(TableGroupOrder { target, order });
        }

        // Maybe validate that the order "all" in the first position cannot be combined with other orders?

        Some(Self {
            orders: sort_orders,
        })
    }
}

impl std::fmt::Display for TableGroupOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.target, self.order)
    }
}
