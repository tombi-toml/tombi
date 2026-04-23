use std::borrow::Cow;

use tombi_document_tree::{DocumentTree, TableKind, Value, dig_accessors};
use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, ArraySchema, CurrentSchema, ReferableValueSchemas, SchemaAccessor, SchemaContext,
    SchemaItem, TableSchema, ValueSchema,
};

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

pub(crate) async fn resolve_accessors_for_document_or_schema(
    document_tree: &DocumentTree,
    accessors: Vec<Accessor>,
    schema_context: &SchemaContext<'_>,
) -> (Vec<Accessor>, Option<CurrentSchema<'static>>) {
    for depth in (0..=accessors.len()).rev() {
        if let Some(current_schema) =
            resolve_current_schema(&accessors[..depth], schema_context).await
        {
            let mut accessors = accessors;
            accessors.truncate(depth);
            return (accessors, Some(current_schema));
        }
    }

    (align_with_document_tree(document_tree, accessors), None)
}

/// A trailing Array index whose value is a leaf is popped so dispatch
/// happens from the enclosing Array rather than the inner leaf.
fn align_with_document_tree(
    document_tree: &DocumentTree,
    accessors: Vec<Accessor>,
) -> Vec<Accessor> {
    let mut resolved_accessors = accessors;
    loop {
        if resolved_accessors.is_empty() {
            return resolved_accessors;
        }
        match dig_accessors(document_tree, &resolved_accessors) {
            Some((_, value)) => {
                if matches!(resolved_accessors.last(), Some(Accessor::Index(_)))
                    && is_leaf_array_element(value)
                {
                    resolved_accessors.pop();
                }
                return resolved_accessors;
            }
            None => {
                resolved_accessors.pop();
            }
        }
    }
}

fn is_leaf_array_element(value: &Value) -> bool {
    match value {
        Value::Boolean(_)
        | Value::Integer(_)
        | Value::Float(_)
        | Value::String(_)
        | Value::OffsetDateTime(_)
        | Value::LocalDateTime(_)
        | Value::LocalDate(_)
        | Value::LocalTime(_)
        | Value::Incomplete { .. } => true,
        Value::Table(table) => table.kind() == TableKind::KeyValue,
        Value::Array(_) => false,
    }
}

async fn resolve_current_schema(
    accessors: &[Accessor],
    schema_context: &SchemaContext<'_>,
) -> Option<CurrentSchema<'static>> {
    let document_schema = schema_context.root_schema?;
    let value_schema = document_schema.value_schema.as_ref()?;
    let current_schema = CurrentSchema {
        value_schema: value_schema.clone(),
        schema_uri: Cow::Owned(document_schema.schema_uri.clone()),
        definitions: Cow::Owned(document_schema.definitions.clone()),
    };

    resolve_schema_with_accessors(current_schema, accessors, schema_context).await
}

fn resolve_schema_with_accessors<'a: 'b, 'b>(
    current_schema: CurrentSchema<'static>,
    accessors: &'a [Accessor],
    schema_context: &'a SchemaContext<'a>,
) -> tombi_future::BoxFuture<'b, Option<CurrentSchema<'static>>> {
    async move {
        let Some((accessor, remaining_accessors)) = accessors.split_first() else {
            return Some(current_schema);
        };

        let composite_schemas = match current_schema.value_schema.as_ref() {
            ValueSchema::OneOf(schema) => Some(schema.schemas.clone()),
            ValueSchema::AnyOf(schema) => Some(schema.schemas.clone()),
            ValueSchema::AllOf(schema) => Some(schema.schemas.clone()),
            _ => None,
        };
        if let Some(schemas) = composite_schemas {
            return resolve_composite_schema_with_accessors(
                &schemas,
                current_schema,
                accessors,
                schema_context,
            )
            .await;
        }

        match (accessor, current_schema.value_schema.as_ref()) {
            (Accessor::Key(_), ValueSchema::Table(table_schema)) => {
                let next_schema = table_schema
                    .resolve_property_schema(
                        &SchemaAccessor::from(accessor),
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                    )
                    .await
                    .inspect_err(|err| log::warn!("{err}"))
                    .ok()
                    .flatten()?
                    .into_owned();
                resolve_schema_with_accessors(next_schema, remaining_accessors, schema_context)
                    .await
            }
            (Accessor::Index(index), ValueSchema::Array(array_schema)) => {
                let next_schema = resolve_array_item_schema(
                    *index,
                    array_schema,
                    &current_schema,
                    schema_context,
                )
                .await?
                .into_owned();
                resolve_schema_with_accessors(next_schema, remaining_accessors, schema_context)
                    .await
            }
            _ => None,
        }
    }
    .boxed()
}

fn resolve_composite_schema_with_accessors<'a: 'b, 'b>(
    schemas: &'a ReferableValueSchemas,
    current_schema: CurrentSchema<'static>,
    accessors: &'a [Accessor],
    schema_context: &'a SchemaContext<'a>,
) -> tombi_future::BoxFuture<'b, Option<CurrentSchema<'static>>> {
    async move {
        let schema_visits = tombi_schema_store::SchemaVisits::default();
        let collected = tombi_schema_store::resolve_and_collect_schemas(
            schemas,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
            &schema_visits,
            accessors,
        )
        .await?;

        for schema in collected {
            if let Some(resolved) =
                resolve_schema_with_accessors(schema.into_owned(), accessors, schema_context).await
            {
                return Some(resolved);
            }
        }

        None
    }
    .boxed()
}

pub fn remaining_keys<'a>(
    keys: &'a [tombi_document_tree::Key],
    accessors: &[Accessor],
) -> &'a [tombi_document_tree::Key] {
    let resolved_key_count = accessors
        .iter()
        .filter(|accessor| accessor.as_key().is_some())
        .count();
    &keys[resolved_key_count.min(keys.len())..]
}
