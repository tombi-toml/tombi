use std::borrow::Cow;

use itertools::Itertools;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, DocumentSchema, OneOfSchema, PropertySchema,
    SchemaAccessor, ValueSchema,
};
use tombi_validator::Validate;

use crate::edit::EditRecursive;

use super::get_value_schema;

impl crate::Edit<tombi_document_tree::Table> for tombi_ast::KeyValue {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Table,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let Some(keys_accessors) = self.get_accessors(schema_context.toml_version) else {
                return Vec::with_capacity(0);
            };

            let accessors = accessors
                .iter()
                .cloned()
                .chain(keys_accessors)
                .collect_vec();

            let mut changes = vec![];

            let document_tree_value = tombi_document_tree::Value::Table(
                self.clone()
                    .into_document_tree_and_errors(schema_context.toml_version)
                    .tree,
            );

            if let Some(current_schema) = current_schema {
                if let Some(value_schema) = get_value_schema(
                    &document_tree_value,
                    &accessors,
                    current_schema,
                    schema_context,
                )
                .await
                {
                    if let Some(value) = self.value() {
                        changes.extend(
                            value
                                .edit(
                                    &tombi_document_tree::Value::Table(node.clone()),
                                    &accessors,
                                    source_path,
                                    Some(&CurrentSchema {
                                        value_schema: Cow::Owned(value_schema),
                                        schema_uri: current_schema.schema_uri.clone(),
                                        definitions: current_schema.definitions.clone(),
                                    }),
                                    schema_context,
                                )
                                .await,
                        );
                        return changes;
                    }
                }
            }

            if let Some(value) = self.value() {
                changes.extend(
                    value
                        .edit(
                            &tombi_document_tree::Value::Table(node.clone()),
                            &accessors,
                            source_path,
                            None,
                            schema_context,
                        )
                        .await,
                );
            }

            changes
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Value {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        // edit_fn: impl FnOnce(&'a tombi_document_tree::Value) -> BoxFuture<'b, Vec<crate::Change>>
        //     + std::marker::Send
        //     + 'b,
        key_accessors: &'a [Accessor],
        accessors: &'a [Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            if let Some(Ok(DocumentSchema {
                value_schema: Some(value_schema),
                schema_uri,
                definitions,
                ..
            })) = schema_context
                .get_subschema(accessors.as_ref(), current_schema)
                .await
            {
                return self
                    .edit_recursive(
                        // edit_fn,
                        key_accessors,
                        accessors,
                        Some(&CurrentSchema {
                            value_schema: Cow::Borrowed(&value_schema),
                            schema_uri: Cow::Borrowed(&schema_uri),
                            definitions: Cow::Borrowed(&definitions),
                        }),
                        schema_context,
                    )
                    .await;
            }

            match (key_accessors.as_ref().first(), self) {
                (Some(Accessor::Key(key_str)), tombi_document_tree::Value::Table(table)) => {
                    let key_accessor = Accessor::Key(key_str.to_owned());

                    table
                        .edit_recursive(
                            // edit_fn,
                            &key_accessors[1..],
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(key_accessor))
                                .collect_vec(),
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                (Some(Accessor::Index(index)), tombi_document_tree::Value::Array(array)) => {
                    let index_accessor = Accessor::Index(*index);

                    array
                        .edit_recursive(
                            // edit_fn,
                            &key_accessors[1..],
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(index_accessor))
                                .collect_vec(),
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                (None, _) => Vec::with_capacity(0),
                _ => Vec::with_capacity(0),
            }
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Table {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        // edit_fn: impl FnOnce(&'a tombi_document_tree::Value) -> BoxFuture<'b, Vec<crate::Change>>
        //     + std::marker::Send
        //     + 'b,
        key_accessors: &'a [Accessor],
        accessors: &'a [Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let Some(Accessor::Key(accessor_str)) = key_accessors.as_ref().first() else {
                return Vec::with_capacity(0);
            };
            let Some(value) = self.get(accessor_str) else {
                return Vec::with_capacity(0);
            };

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Table(table_schema) => {
                        let key_accessor = SchemaAccessor::Key(accessor_str.to_owned());

                        let mut properties = table_schema.properties.write().await;

                        if let Some(PropertySchema {
                            property_schema, ..
                        }) = properties.get_mut(&key_accessor)
                        {
                            if let Ok(Some(current_schema)) = property_schema
                                .resolve(
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                return value
                                    .edit_recursive(
                                        // edit_fn,
                                        key_accessors,
                                        accessors,
                                        Some(&current_schema),
                                        schema_context,
                                    )
                                    .await;
                            }
                        }
                        if let Some(pattern_properties) = &table_schema.pattern_properties {
                            for (
                                property_key,
                                PropertySchema {
                                    property_schema, ..
                                },
                            ) in pattern_properties.write().await.iter_mut()
                            {
                                let Ok(pattern) = regex::Regex::new(property_key) else {
                                    tracing::warn!(
                                        "Invalid regex pattern property: {}",
                                        property_key
                                    );
                                    continue;
                                };
                                if pattern.is_match(accessor_str) {
                                    tracing::trace!(
                                        "pattern_property_schema = {:?}",
                                        &property_schema
                                    );
                                    if let Ok(Some(current_schema)) = property_schema
                                        .resolve(
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        return value
                                            .edit_recursive(
                                                // edit_fn,
                                                key_accessors,
                                                accessors,
                                                Some(&current_schema),
                                                schema_context,
                                            )
                                            .await;
                                    }
                                }
                            }
                        }
                        if let Some((_, referable_additional_property_schema)) =
                            &table_schema.additional_property_schema
                        {
                            tracing::trace!(
                                "additional_property_schema = {:?}",
                                referable_additional_property_schema
                            );
                            if let Ok(Some(current_schema)) = referable_additional_property_schema
                                .write()
                                .await
                                .resolve(
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                return value
                                    .edit_recursive(
                                        // edit_fn,
                                        key_accessors,
                                        accessors,
                                        Some(&current_schema),
                                        schema_context,
                                    )
                                    .await;
                            }
                        }
                    }
                    ValueSchema::AllOf(AllOfSchema { schemas, .. })
                    | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                    | ValueSchema::OneOf(OneOfSchema { schemas, .. }) => {
                        for referable_schema in schemas.write().await.iter_mut() {
                            if let Ok(Some(current_schema)) = referable_schema
                                .resolve(
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                if self
                                    .validate(accessors, Some(&current_schema), schema_context)
                                    .await
                                    .is_ok()
                                {
                                    return value
                                        .edit_recursive(
                                            // edit_fn,
                                            key_accessors,
                                            accessors,
                                            Some(&current_schema),
                                            schema_context,
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            value
                .edit_recursive(
                    //edit_fn,
                    key_accessors,
                    accessors,
                    None,
                    schema_context,
                )
                .await
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Array {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        // edit_fn: impl FnOnce(&'a tombi_document_tree::Value) -> BoxFuture<'b, Vec<crate::Change>>
        //     + std::marker::Send
        //     + 'b,
        key_accessors: &'a [Accessor],
        accessors: &'a [Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            let Some(Accessor::Index(index)) = key_accessors.as_ref().first() else {
                return Vec::with_capacity(0);
            };
            let Some(value) = self.get(*index) else {
                return Vec::with_capacity(0);
            };

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Array(array_schema) => {
                        if let Some(items) = &array_schema.items {
                            let mut item_schema = items.write().await;

                            if let Ok(Some(current_schema)) = item_schema
                                .resolve(
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                return value
                                    .edit_recursive(
                                        // edit_fn,
                                        key_accessors,
                                        accessors,
                                        Some(&current_schema),
                                        schema_context,
                                    )
                                    .await;
                            }
                        }
                    }
                    ValueSchema::AllOf(AllOfSchema { schemas, .. })
                    | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                    | ValueSchema::OneOf(OneOfSchema { schemas, .. }) => {
                        for referable_schema in schemas.write().await.iter_mut() {
                            if let Ok(Some(current_schema)) = referable_schema
                                .resolve(
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                if self
                                    .validate(accessors, Some(&current_schema), schema_context)
                                    .await
                                    .is_ok()
                                {
                                    return value
                                        .edit_recursive(
                                            // edit_fn,
                                            key_accessors,
                                            accessors,
                                            Some(&current_schema),
                                            schema_context,
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            value
                .edit_recursive(
                    // edit_fn,
                    &key_accessors,
                    &accessors,
                    None,
                    schema_context,
                )
                .await
        }
        .boxed()
    }
}
