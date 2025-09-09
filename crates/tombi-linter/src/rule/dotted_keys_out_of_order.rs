use crate::Rule;
use ahash::AHashMap;
use itertools::Itertools;
use tombi_comment_directive::value::TableCommonRules;
use tombi_config::SeverityLevel;
use tombi_severity_level::SeverityLevelDefaultWarn;
use tombi_validator::comment_directive::get_tombi_key_value_rules_and_diagnostics;

pub struct DottedKeysOutOfOrderRule;

impl Rule<tombi_ast::Root> for DottedKeysOutOfOrderRule {
    async fn check(root: &tombi_ast::Root, l: &mut crate::Linter<'_>) {
        check_dotted_keys_out_of_order(root.key_values(), root.comment_directives(), l).await;
    }
}

impl Rule<tombi_ast::Table> for DottedKeysOutOfOrderRule {
    async fn check(table: &tombi_ast::Table, l: &mut crate::Linter<'_>) {
        check_dotted_keys_out_of_order(table.key_values(), table.comment_directives(), l).await;
    }
}

impl Rule<tombi_ast::ArrayOfTable> for DottedKeysOutOfOrderRule {
    async fn check(table: &tombi_ast::ArrayOfTable, l: &mut crate::Linter<'_>) {
        check_dotted_keys_out_of_order(table.key_values(), table.comment_directives(), l).await;
    }
}

impl Rule<tombi_ast::InlineTable> for DottedKeysOutOfOrderRule {
    async fn check(table: &tombi_ast::InlineTable, l: &mut crate::Linter<'_>) {
        check_dotted_keys_out_of_order(table.key_values(), table.comment_directives(), l).await;
    }
}

async fn check_dotted_keys_out_of_order(
    key_values: impl Iterator<Item = tombi_ast::KeyValue>,
    comment_directives: impl Iterator<Item = tombi_ast::TombiValueCommentDirective>,
    l: &mut crate::Linter<'_>,
) {
    let level = get_tombi_key_value_rules_and_diagnostics::<TableCommonRules>(
        &comment_directives.collect_vec(),
        &[],
    )
    .await
    .0
    .as_ref()
    .map(|rules| &rules.value)
    .and_then(|rules| {
        rules
            .dotted_keys_out_of_order
            .as_ref()
            .map(SeverityLevelDefaultWarn::from)
    })
    .unwrap_or_else(|| {
        l.options()
            .rules
            .as_ref()
            .and_then(|rules| rules.dotted_keys_out_of_order)
            .unwrap_or_default()
    });

    if level == SeverityLevel::Off {
        return;
    }

    let mut prefix_groups: AHashMap<String, Vec<(usize, tombi_text::Range)>> = AHashMap::new();

    // Single pass to collect all data
    for (index, key_value) in key_values.enumerate() {
        if let Some(key_text) = key_value
            .keys()
            .and_then(|keys| keys.keys().next())
            .and_then(|key| key.try_to_raw_text(l.toml_version()).ok())
        {
            prefix_groups
                .entry(key_text)
                .or_default()
                .push((index, key_value.range()));
        }
    }

    // Check if any prefix group is out of order
    let mut out_of_order_ranges = Vec::new();

    for (_, positions) in &prefix_groups {
        if positions
            .windows(2)
            .any(|window| window[0].0 + 1 != window[1].0)
        {
            out_of_order_ranges.extend(positions.iter().map(|(_, range)| *range))
        }
    }

    // Report diagnostics for all out-of-order dotted keys
    if !out_of_order_ranges.is_empty() {
        for range in out_of_order_ranges {
            l.extend_diagnostics(crate::Diagnostic {
                kind: crate::DiagnosticKind::DottedKeysOutOfOrder,
                level: level.into(),
                range,
            });
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
