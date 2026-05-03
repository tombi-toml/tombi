use itertools::Itertools;
use tombi_ast::{AstNode, GetHeaderAccessors};
use tombi_comment_directive::value::{
    TableCommonFormatRules, TableCommonLintRules, TombiValueDirectiveContent,
};
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::{CurrentSchema, SchemaContext};
use tombi_syntax::SyntaxElement;

use crate::rule::TableOrderOverrides;
use crate::rule::table_keys_order::{get_sorted_accessors, table_keys_order};

pub async fn root_table_keys_order<'a>(
    key_value_groups: Vec<tombi_ast::KeyValueGroup>,
    table_or_array_of_tables: Vec<tombi_ast::TableOrArrayOfTable>,
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a SchemaContext<'a>,
    comment_directive: Option<
        TombiValueDirectiveContent<TableCommonFormatRules, TableCommonLintRules>,
    >,
    table_order_overrides: Option<&TableOrderOverrides>,
) -> Vec<crate::Change> {
    if key_value_groups.is_empty() && table_or_array_of_tables.is_empty() {
        return Vec::with_capacity(0);
    }

    if comment_directive
        .as_ref()
        .and_then(|c| c.table_keys_order_disabled())
        .unwrap_or_default()
    {
        return Vec::with_capacity(0);
    }
    let comment_directive_order = comment_directive
        .as_ref()
        .and_then(|comment_directive| comment_directive.table_keys_order().map(Into::into));

    let mut changes = Vec::new();
    for key_value_group in key_value_groups {
        let key_values_with_comma = key_value_group.key_values_with_comma().collect_vec();
        if key_values_with_comma.is_empty() {
            continue;
        }

        changes.extend(
            table_keys_order(
                &tombi_document_tree::Value::Table(
                    key_values_with_comma
                        .iter()
                        .map(|(kv, _)| kv.clone())
                        .collect_vec()
                        .into_document_tree_and_errors(schema_context.toml_version)
                        .tree,
                ),
                &[],
                key_values_with_comma,
                current_schema,
                schema_context,
                comment_directive.clone(),
            )
            .await,
        );
    }

    if table_or_array_of_tables.is_empty() {
        return changes;
    }

    let schema_override = schema_context.table_order_override(current_schema, &[]);
    let root_order =
        comment_directive_order.or(schema_override.and_then(|override_item| override_item.order));
    let schema_order_enabled = schema_override.is_some_and(|override_item| !override_item.disabled)
        || schema_context.schema_table_keys_order_enabled(current_schema);

    if root_order.is_none() && !schema_order_enabled {
        return changes;
    }

    let old = std::ops::RangeInclusive::new(
        SyntaxElement::Node(table_or_array_of_tables.first().unwrap().syntax().clone()),
        SyntaxElement::Node(table_or_array_of_tables.last().unwrap().syntax().clone()),
    );

    let Some(sorted_table) = get_sorted_accessors(
        &tombi_document_tree::Value::Table(
            table_or_array_of_tables
                .clone()
                .into_document_tree_and_errors(schema_context.toml_version)
                .tree,
        ),
        &[],
        table_or_array_of_tables
            .into_iter()
            .map(|table| {
                (
                    table
                        .get_header_accessors(schema_context.toml_version)
                        .unwrap_or_default(),
                    table,
                )
            })
            .collect_vec(),
        current_schema,
        schema_context,
        root_order,
        table_order_overrides,
    )
    .await
    else {
        return changes;
    };

    let new = sorted_table
        .into_iter()
        .map(|kv| SyntaxElement::Node(kv.syntax().clone()))
        .collect_vec();

    changes.push(crate::Change::ReplaceRange { old, new });

    changes
}
