use itertools::Itertools;
use tombi_syntax::SyntaxElement;
use tombi_ast::AstNode;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::{CurrentSchema, SchemaAccessor, SchemaContext};

use crate::rule::table_keys_order::{sorted_accessors, table_keys_order};

pub async fn root_table_keys_order<'a>(
    key_values: Vec<tombi_ast::KeyValue>,
    table_or_array_of_tables: Vec<tombi_ast::TableOrArrayOfTable>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
) -> Vec<crate::Change> {
    if key_values.is_empty() && table_or_array_of_tables.is_empty() {
        return Vec::with_capacity(0);
    }

    let mut changes = table_keys_order(
        &tombi_document_tree::Value::Table(
            key_values
                .clone()
                .into_document_tree_and_errors(schema_context.toml_version)
                .tree,
        ),
        key_values,
        current_schema,
        schema_context,
    )
    .await;

    if table_or_array_of_tables.is_empty() {
        return changes;
    }

    let old = std::ops::RangeInclusive::new(
        SyntaxElement::Node(table_or_array_of_tables.first().unwrap().syntax().clone()),
        SyntaxElement::Node(table_or_array_of_tables.last().unwrap().syntax().clone()),
    );

    let targets = table_or_array_of_tables
        .clone()
        .into_iter()
        .map(|table| {
            (
                table
                    .header()
                    .map(|key| {
                        key.keys()
                            .map(|key| {
                                SchemaAccessor::Key(
                                    key.try_to_raw_text(schema_context.toml_version).unwrap(),
                                )
                            })
                            .collect_vec()
                    })
                    .unwrap_or_default(),
                table,
            )
        })
        .collect_vec();

    let new = sorted_accessors(
        &tombi_document_tree::Value::Table(
            table_or_array_of_tables
                .into_document_tree_and_errors(schema_context.toml_version)
                .tree,
        ),
        &[],
        targets,
        current_schema,
        schema_context,
    )
    .await
    .into_iter()
    .map(|kv| SyntaxElement::Node(kv.syntax().clone()))
    .collect_vec();

    changes.push(crate::Change::ReplaceRange { old, new });

    changes
}
