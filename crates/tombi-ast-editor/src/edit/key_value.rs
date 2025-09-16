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
        value: &'a tombi_ast::Value,
        key_accessors: &'a [Accessor],
        accessors: &'a [Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            match (key_accessors.first(), self) {
                (Some(Accessor::Key(key_str)), tombi_document_tree::Value::Table(table)) => {
                    table
                        .edit_recursive(
                            &key_accessors[1..],
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(Accessor::Key(key_str.to_owned())))
                                .collect_vec(),
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                (Some(Accessor::Index(index)), tombi_document_tree::Value::Array(array)) => {
                    array
                        .edit_recursive(
                            &key_accessors[1..],
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(Accessor::Index(*index)))
                                .collect_vec(),
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                (None, value) => value.edit(),
                _ => Vec::with_capacity(0),
            }
        }
        .boxed()
    }
}

impl EditRecursive for tombi_ast::KeyValue {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
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
                .get_subschema(accessors, current_schema)
                .await
            {
                return self
                    .edit_recursive(
                        node,
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

            if let Some(Accessor::Key(accessor_str)) = accessors.first() {
                let Some(value) = self.get(accessor_str) else {
                    value.edit_recursive(
                        key_accessors,
                        accessors,
                        Some(&CurrentSchema {
                            value_schema: Cow::Borrowed(&value_schema),
                            schema_uri: Cow::Borrowed(&schema_uri),
                            definitions: Cow::Borrowed(&definitions),
                        }),
                        schema_context,
                    )
                };
            }

            Vec::with_capacity(0)
        }
        .boxed()
    }
}

impl EditRecursive for tombi_document_tree::Array {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
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
                .get_subschema(accessors, current_schema)
                .await
            {
                return self
                    .edit_recursive(
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

            Vec::with_capacity(0)
        }
        .boxed()
    }
}
