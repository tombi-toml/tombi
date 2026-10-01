use crate::Rule;
use tombi_comment_directive::value::{TableCommonFormatRules, TableCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_config::SeverityLevel;
use tombi_severity_level::SeverityLevelDefaultWarn;

pub struct DottedKeysOutOfOrderRule;

impl Rule<tombi_ast_syntax::Root<'_>> for DottedKeysOutOfOrderRule {
    async fn check(root: &tombi_ast_syntax::Root<'_>, l: &mut crate::Linter<'_>) {
        check_dotted_keys_out_of_order(root.key_values(), root.comment_directives(), l).await;
    }
}

impl Rule<tombi_ast_syntax::Table<'_>> for DottedKeysOutOfOrderRule {
    async fn check(table: &tombi_ast_syntax::Table<'_>, l: &mut crate::Linter<'_>) {
        check_dotted_keys_out_of_order(table.key_values(), table.comment_directives(), l).await;
    }
}

impl Rule<tombi_ast_syntax::ArrayOfTable<'_>> for DottedKeysOutOfOrderRule {
    async fn check(table: &tombi_ast_syntax::ArrayOfTable<'_>, l: &mut crate::Linter<'_>) {
        check_dotted_keys_out_of_order(table.key_values(), table.comment_directives(), l).await;
    }
}

impl Rule<tombi_ast_syntax::InlineTable<'_>> for DottedKeysOutOfOrderRule {
    async fn check(table: &tombi_ast_syntax::InlineTable<'_>, l: &mut crate::Linter<'_>) {
        check_dotted_keys_out_of_order(table.key_values(), table.comment_directives(), l).await;
    }
}

async fn check_dotted_keys_out_of_order(
    key_values: impl Iterator<Item = tombi_ast_syntax::KeyValue<'_>>,
    comment_directives: impl Iterator<Item = tombi_ast_syntax::TombiValueCommentDirective>,
    l: &mut crate::Linter<'_>,
) {
    let comment_directive = get_comment_directive_content::<
        TableCommonFormatRules,
        TableCommonLintRules,
    >(comment_directives);

    if comment_directive
        .as_ref()
        .and_then(|comment_directive| comment_directive.table_keys_order_disabled())
        .unwrap_or_default()
    {
        return;
    }

    let level = comment_directive
        .as_ref()
        .and_then(|comment_directive| comment_directive.lint_rules())
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

    let mut prefix_groups: tombi_hashmap::HashMap<String, Vec<(usize, tombi_text::Span)>> =
        tombi_hashmap::HashMap::new();

    // Single pass to collect all data
    for (index, key_value) in key_values.enumerate() {
        let Some(keys) = key_value.keys() else {
            continue;
        };
        let Some(key) = keys.keys().next() else {
            continue;
        };
        let Ok(content) = key.try_to_content(l.toml_version()) else {
            continue;
        };
        let offset = (index, key_value.span());

        if let Some(offsets) = prefix_groups.get_mut(content.as_ref()) {
            offsets.push(offset);
        } else {
            prefix_groups.insert(content.into_owned(), vec![offset]);
        }
    }

    // Check if any prefix group is out of order
    let mut out_of_order_spans = Vec::new();

    for (_, offsets) in &prefix_groups {
        if offsets
            .windows(2)
            .any(|window| window[0].0 + 1 != window[1].0)
        {
            out_of_order_spans.extend(offsets.iter().map(|(_, span)| *span))
        }
    }

    // Report diagnostics for all out-of-order dotted keys
    if !out_of_order_spans.is_empty() {
        for span in out_of_order_spans {
            l.extend_diagnostics(crate::Diagnostic {
                kind: crate::DiagnosticKind::DottedKeysOutOfOrder,
                level: level.into(),
                span,
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
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message() == "defining dotted keys out-of-order is discouraged")
        );
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
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message() == "defining dotted keys out-of-order is discouraged")
        );
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
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message() == "defining dotted keys out-of-order is discouraged")
        );
    }
}
