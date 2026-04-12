use tombi_schema_store::{ArraySchema, CurrentSchema, SchemaContext, SchemaItem, TableSchema};

async fn resolve_schema_item_owned(
    schema_item: &SchemaItem,
    current_schema: &CurrentSchema<'_>,
    schema_context: &SchemaContext<'_>,
) -> Option<CurrentSchema<'static>> {
    tombi_schema_store::resolve_schema_item(
        schema_item,
        current_schema.schema_uri.clone(),
        current_schema.definitions.clone(),
        schema_context.store,
    )
    .await
    .inspect_err(|err| log::warn!("{err}"))
    .ok()
    .flatten()
    .map(CurrentSchema::into_owned)
}

pub(crate) async fn resolve_array_item_schema(
    index: usize,
    array_schema: &ArraySchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &SchemaContext<'_>,
) -> Option<CurrentSchema<'static>> {
    if let Some(prefix_items) = &array_schema.prefix_items {
        if let Some(schema_item) = prefix_items.get(index) {
            return resolve_schema_item_owned(schema_item, current_schema, schema_context).await;
        }

        if let Some(schema_item) = &array_schema.additional_items_schema {
            return resolve_schema_item_owned(schema_item, current_schema, schema_context).await;
        }

        if array_schema.additional_items == Some(false) {
            return None;
        }

        if let Some(schema_item) = &array_schema.items {
            return resolve_schema_item_owned(schema_item, current_schema, schema_context).await;
        }
    } else if let Some(schema_item) = &array_schema.items {
        return resolve_schema_item_owned(schema_item, current_schema, schema_context).await;
    }

    if let Some(schema_item) = &array_schema.unevaluated_items_schema {
        return resolve_schema_item_owned(schema_item, current_schema, schema_context).await;
    }

    None
}

pub(crate) async fn resolve_table_unevaluated_property_schema(
    table_schema: &TableSchema,
    current_schema: &CurrentSchema<'_>,
    schema_context: &SchemaContext<'_>,
) -> Option<CurrentSchema<'static>> {
    let Some(schema_item) = &table_schema.unevaluated_property_schema else {
        return None;
    };

    resolve_schema_item_owned(schema_item, current_schema, schema_context).await
}
