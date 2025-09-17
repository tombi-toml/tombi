use std::borrow::Cow;

use itertools::Itertools;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{Accessor, CurrentSchema, DocumentSchema};

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
        edit_fn: impl FnOnce(&'a tombi_document_tree::Value) -> BoxFuture<'b, Vec<crate::Change>>
            + std::marker::Send
            + 'b,
        key_accessors: Cow<'a, [Accessor]>,
        accessors: Cow<'a, [Accessor]>,
        current_schema: Option<tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            if let Some(Ok(DocumentSchema {
                value_schema: Some(value_schema),
                schema_uri,
                definitions,
                ..
            })) = schema_context
                .get_subschema(accessors.as_ref(), current_schema.as_ref())
                .await
            {
                let next_schema = CurrentSchema {
                    value_schema: Cow::Owned(value_schema),
                    schema_uri: Cow::Owned(schema_uri),
                    definitions: Cow::Owned(definitions),
                };

                return self
                    .edit_recursive(
                        edit_fn,
                        key_accessors.clone(),
                        accessors.clone(),
                        Some(next_schema),
                        schema_context,
                    )
                    .await;
            }

            match (key_accessors.as_ref().first(), self) {
                (Some(Accessor::Key(key_str)), tombi_document_tree::Value::Table(table)) => {
                    let mut extended_accessors = accessors.as_ref().to_vec();
                    extended_accessors.push(Accessor::Key(key_str.to_owned()));

                    let next_key_accessors = Cow::Owned(key_accessors.as_ref()[1..].to_vec());

                    table
                        .edit_recursive(
                            edit_fn,
                            next_key_accessors,
                            Cow::Owned(extended_accessors),
                            current_schema.clone(),
                            schema_context,
                        )
                        .await
                }
                (Some(Accessor::Index(index)), tombi_document_tree::Value::Array(array)) => {
                    let mut extended_accessors = accessors.as_ref().to_vec();
                    extended_accessors.push(Accessor::Index(*index));

                    let next_key_accessors = Cow::Owned(key_accessors.as_ref()[1..].to_vec());

                    array
                        .edit_recursive(
                            edit_fn,
                            next_key_accessors,
                            Cow::Owned(extended_accessors),
                            current_schema.clone(),
                            schema_context,
                        )
                        .await
                }
                (None, value) => edit_fn(value).await,
                _ => Vec::with_capacity(0),
            }
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Table {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        edit_fn: impl FnOnce(&'a tombi_document_tree::Value) -> BoxFuture<'b, Vec<crate::Change>>
            + std::marker::Send
            + 'b,
        key_accessors: Cow<'a, [Accessor]>,
        accessors: Cow<'a, [Accessor]>,
        current_schema: Option<tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            if let Some(Accessor::Key(accessor_str)) = accessors.as_ref().first() {
                let Some(value) = self.get(accessor_str) else {
                    return Vec::with_capacity(0);
                };

                if let Some(current_schema) = current_schema.as_ref() {
                    match current_schema.value_schema.as_ref() {
                        _ => {}
                    }
                }

                return value
                    .edit_recursive(edit_fn, key_accessors, accessors, None, schema_context)
                    .await;
            }

            Vec::with_capacity(0)
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Array {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        edit_fn: impl FnOnce(&'a tombi_document_tree::Value) -> BoxFuture<'b, Vec<crate::Change>>
            + std::marker::Send
            + 'b,
        key_accessors: Cow<'a, [Accessor]>,
        accessors: Cow<'a, [Accessor]>,
        current_schema: Option<tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            if let Some(Ok(DocumentSchema {
                value_schema: Some(value_schema),
                schema_uri,
                definitions,
                ..
            })) = schema_context
                .get_subschema(accessors.as_ref(), current_schema.as_ref())
                .await
            {
                let next_schema = CurrentSchema {
                    value_schema: Cow::Owned(value_schema),
                    schema_uri: Cow::Owned(schema_uri),
                    definitions: Cow::Owned(definitions),
                };

                return self
                    .edit_recursive(
                        edit_fn,
                        key_accessors.clone(),
                        accessors.clone(),
                        Some(next_schema),
                        schema_context,
                    )
                    .await;
            }

            Vec::with_capacity(0)
        }
        .boxed()
    }
}
