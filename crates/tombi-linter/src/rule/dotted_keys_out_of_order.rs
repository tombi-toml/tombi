use ahash::AHashMap;
use itertools::Itertools;
use tombi_ast::AstNode;

use crate::Rule;

pub struct DottedKeysOutOfOrderRule;

impl Rule<tombi_ast::Root> for DottedKeysOutOfOrderRule {
    fn check(node: &tombi_ast::Root, l: &mut crate::Linter) {
        let source_text = l.source_text();
        let mut prefix_groups: AHashMap<&str, Vec<(usize, tombi_text::Range)>> = AHashMap::new();
        let mut all_positions = Vec::new();

        // Single pass to collect all data
        for (position, item) in node.items().enumerate() {
            if let tombi_ast::RootItem::KeyValue(key_value) = item {
                all_positions.push(position);

                if let Some(keys) = key_value.keys() {
                    let key_parts = keys
                        .keys()
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
                        .collect_vec();

                    if key_parts.len() > 1 {
                        // This is a dotted key
                        let prefix = key_parts[0];
                        prefix_groups
                            .entry(prefix)
                            .or_insert_with(Vec::new)
                            .push((position, key_value.syntax().range()));
                    }
                }
            }
        }

        // Check if any prefix group is out of order
        let mut out_of_order_ranges = Vec::new();

        for (_, positions) in &prefix_groups {
            if positions.len() > 1 {
                let min_pos = positions.iter().map(|(pos, _)| *pos).min().unwrap();
                let max_pos = positions.iter().map(|(pos, _)| *pos).max().unwrap();

                // Count keys in the range (including non-dotted)
                let keys_in_range = all_positions
                    .iter()
                    .filter(|&&pos| pos >= min_pos && pos <= max_pos)
                    .count();

                // If there are more keys in range than dotted keys with this prefix, it's out of order
                if keys_in_range > positions.len() {
                    out_of_order_ranges.extend(positions.iter().map(|(_, range)| *range));
                }
            }
        }

        // Report diagnostics for all out-of-order dotted keys
        if !out_of_order_ranges.is_empty() {
            let level = l
                .options()
                .rules
                .as_ref()
                .and_then(|rules| rules.dotted_keys_out_of_order)
                .unwrap_or_default()
                .into();

            for range in out_of_order_ranges {
                l.extend_diagnostics(crate::Severity {
                    kind: crate::SeverityKind::DottedKeysOutOfOrder,
                    level,
                    range,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_dotted_keys_out_of_order() {
        let source = r#"
apple.type = "fruit"
orange.type = "fruit"

apple.skin = "thin"
orange.skin = "thick"

apple.color = "red"
orange.color = "orange"
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

        // Should warn on ALL 6 keys when out of order is detected
        assert_eq!(diagnostics.len(), 6);

        // All warnings should have the same message
        assert!(diagnostics
            .iter()
            .all(|d| d.message() == "Defining dotted keys out-of-order is discouraged"));
    }

    #[tokio::test]
    async fn test_dotted_keys_in_order() {
        let source = r#"
apple.type = "fruit"
apple.skin = "thin"
apple.color = "red"

orange.type = "fruit"
orange.skin = "thick"
orange.color = "orange"
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
    async fn test_dotted_keys_with_non_dotted_between() {
        let source = r#"
lsp.code-action.enabled = true
toml-version = "v1.0.0"
lsp.completion.enabled = true
exclude = ["node_modules/**/*"]
lsp.formatting.enabled = true
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

        // Should warn on all lsp keys when out of order is detected
        assert_eq!(diagnostics.len(), 3);

        // The warning should have the correct message
        assert!(diagnostics
            .iter()
            .all(|d| d.message() == "Defining dotted keys out-of-order is discouraged"));
    }

    #[tokio::test]
    async fn test_dotted_keys_after_non_dotted_in_order() {
        let source = r#"
toml-version = "v1.0.0"
exclude = ["node_modules/**/*"]
lsp.code-action.enabled = true
lsp.completion.enabled = true
lsp.formatting.enabled = true
"#;

        let result = crate::Linter::new(
            tombi_config::TomlVersion::default(),
            &crate::LintOptions::default(),
            None,
            &tombi_schema_store::SchemaStore::new(),
        )
        .lint(source)
        .await;

        // Should not produce any warnings since dotted keys are grouped together
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dotted_keys_interrupted_by_non_dotted() {
        let source = r#"
toml-version = "v1.0.0"
lsp.code-action.enabled = true
exclude = ["node_modules/**/*"]
lsp.completion.enabled = true
lsp.formatting.enabled = true
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

        // Should warn on all lsp keys when interrupted by non-dotted key
        assert_eq!(diagnostics.len(), 3);

        // All warnings should have the same message
        assert!(diagnostics
            .iter()
            .all(|d| d.message() == "Defining dotted keys out-of-order is discouraged"));
    }
}
