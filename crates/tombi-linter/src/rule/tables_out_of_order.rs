use ahash::AHashMap;
use tombi_ast::AstNode;

use crate::Rule;

pub struct TablesOutOfOrderRule;

impl Rule<tombi_ast::Root> for TablesOutOfOrderRule {
    fn check(node: &tombi_ast::Root, l: &mut crate::Linter) {
        let source_text = l.source_text();
        let mut table_positions: Vec<(usize, Vec<&str>, tombi_text::Range)> = Vec::new();

        // Collect all table definitions
        for (position, item) in node.items().enumerate() {
            match item {
                tombi_ast::RootItem::Table(table) => {
                    if let Some(header) = table.header() {
                        let key_parts = extract_key_parts(&header, source_text);
                        if !key_parts.is_empty() {
                            table_positions.push((position, key_parts, table.syntax().range()));
                        }
                    }
                }
                tombi_ast::RootItem::ArrayOfTable(array_table) => {
                    if let Some(header) = array_table.header() {
                        let key_parts = extract_key_parts(&header, source_text);
                        if !key_parts.is_empty() {
                            table_positions.push((position, key_parts, array_table.syntax().range()));
                        }
                    }
                }
                _ => {}
            }
        }

        // Check if tables with same prefix are out of order
        let mut out_of_order_ranges = Vec::new();
        
        // Group tables by their first key component (prefix)
        let mut prefix_groups: AHashMap<&str, Vec<(usize, tombi_text::Range)>> = AHashMap::new();
        for (pos, keys, range) in &table_positions {
            if !keys.is_empty() {
                prefix_groups
                    .entry(keys[0])
                    .or_insert_with(Vec::new)
                    .push((*pos, *range));
            }
        }

        // For each prefix group with multiple tables, check if they are interrupted
        for (prefix, positions) in &prefix_groups {
            if positions.len() > 1 {
                let min_pos = positions.iter().map(|(pos, _)| *pos).min().unwrap();
                let max_pos = positions.iter().map(|(pos, _)| *pos).max().unwrap();

                // Check if there are any tables with different prefixes between min and max
                let has_interrupting_tables = table_positions
                    .iter()
                    .any(|(pos, keys, _)| {
                        *pos > min_pos && *pos < max_pos && 
                        !keys.is_empty() && keys[0] != *prefix
                    });

                // If there are interrupting tables, mark all tables in this group as out of order
                if has_interrupting_tables {
                    out_of_order_ranges.extend(positions.iter().map(|(_, range)| *range));
                }
            }
        }

        // Report diagnostics for all out-of-order tables
        if !out_of_order_ranges.is_empty() {
            let level = l
                .options()
                .rules
                .as_ref()
                .and_then(|rules| rules.tables_out_of_order)
                .unwrap_or_default()
                .into();

            for range in out_of_order_ranges {
                l.extend_diagnostics(crate::Severity {
                    kind: crate::SeverityKind::TablesOutOfOrder,
                    level,
                    range,
                });
            }
        }
    }
}

fn extract_key_parts<'a>(keys: &tombi_ast::Keys, source_text: &'a str) -> Vec<&'a str> {
    keys.keys()
        .filter_map(|key| match key {
            tombi_ast::Key::BareKey(k) => Some(&source_text[k.syntax().span()]),
            tombi_ast::Key::BasicString(k) => {
                let mut span = k.syntax().span();
                // Remove quotes
                span.start += 1;
                span.end -= 1;
                Some(&source_text[span])
            }
            tombi_ast::Key::LiteralString(k) => {
                let mut span = k.syntax().span();
                // Remove quotes
                span.start += 1;
                span.end -= 1;
                Some(&source_text[span])
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_tables_out_of_order() {
        let source = r#"
[fruit.apple]
color = "red"

[animal]
type = "mammal"

[fruit.orange]
color = "orange"
"#;

        let diagnostics = crate::Linter::new(
            tombi_config::TomlVersion::default(),
            &crate::LintOptions::default(),
            None,
            &tombi_schema_store::SchemaStore::new(),
        )
        .lint(source)
        .await
        .unwrap_err();

        // Should warn on both fruit tables when out of order is detected
        assert_eq!(diagnostics.len(), 2);

        // All warnings should have the same message
        assert!(diagnostics
            .iter()
            .all(|d| d.message() == "Defining tables out-of-order is discouraged"));
    }

    #[tokio::test]
    async fn test_tables_in_order() {
        let source = r#"
[fruit.apple]
color = "red"

[fruit.orange]
color = "orange"

[animal]
type = "mammal"
"#;

        let result = crate::Linter::new(
            tombi_config::TomlVersion::default(),
            &crate::LintOptions::default(),
            None,
            &tombi_schema_store::SchemaStore::new(),
        )
        .lint(source)
        .await;

        // Should not produce any warnings
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_array_tables_out_of_order() {
        let source = r#"
[[products]]
name = "Hammer"

[store]
name = "Hardware Store"

[[products]]
name = "Nail"
"#;

        let diagnostics = crate::Linter::new(
            tombi_config::TomlVersion::default(),
            &crate::LintOptions::default(),
            None,
            &tombi_schema_store::SchemaStore::new(),
        )
        .lint(source)
        .await
        .unwrap_err();

        // Should warn on both products tables when out of order is detected
        assert_eq!(diagnostics.len(), 2);

        // All warnings should have the same message
        assert!(diagnostics
            .iter()
            .all(|d| d.message() == "Defining tables out-of-order is discouraged"));
    }

    #[tokio::test]
    async fn test_mixed_tables_and_array_tables() {
        let source = r#"
[fruit]
name = "fruit section"

[[fruit.items]]
name = "apple"

[vegetable]
name = "vegetable section"

[[fruit.items]]
name = "orange"
"#;

        let diagnostics = crate::Linter::new(
            tombi_config::TomlVersion::default(),
            &crate::LintOptions::default(),
            None,
            &tombi_schema_store::SchemaStore::new(),
        )
        .lint(source)
        .await
        .unwrap_err();

        // Should warn on all fruit-related tables (including [fruit] and both [[fruit.items]])
        assert_eq!(diagnostics.len(), 3);

        // All warnings should have the same message
        assert!(diagnostics
            .iter()
            .all(|d| d.message() == "Defining tables out-of-order is discouraged"));
    }
}