use tombi_ast::AstNode;
use tombi_comment_directive::value::{KeyCommonExtensibleLintRules, KeyFormatRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_config::SeverityLevel;
use tombi_severity_level::SeverityLevelDefaultWarn;

use crate::Rule;

pub struct KeyEmptyRule;

impl Rule<tombi_ast::Root> for KeyEmptyRule {
    async fn check(node: &tombi_ast::Root, l: &mut crate::Linter<'_>) {
        for key_value in node.key_values() {
            if let Some(keys) = key_value.keys() {
                check_key_empty(keys, key_value.comment_directives(), l).await;
            }
        }
    }
}

impl Rule<tombi_ast::Table> for KeyEmptyRule {
    async fn check(node: &tombi_ast::Table, l: &mut crate::Linter<'_>) {
        if let Some(keys) = node.header() {
            check_key_empty(keys, node.header_comment_directives(), l).await;
        }

        for key_value in node.key_values() {
            if let Some(keys) = key_value.keys() {
                check_key_empty(keys, key_value.comment_directives(), l).await;
            }
        }
    }
}

impl Rule<tombi_ast::ArrayOfTable> for KeyEmptyRule {
    async fn check(node: &tombi_ast::ArrayOfTable, l: &mut crate::Linter<'_>) {
        if let Some(keys) = node.header() {
            check_key_empty(keys, node.header_comment_directives(), l).await;
        }

        for key_value in node.key_values() {
            if let Some(keys) = key_value.keys() {
                check_key_empty(keys, key_value.comment_directives(), l).await;
            }
        }
    }
}

impl Rule<tombi_ast::InlineTable> for KeyEmptyRule {
    async fn check(node: &tombi_ast::InlineTable, l: &mut crate::Linter<'_>) {
        for key_value in node.key_values() {
            if let Some(keys) = key_value.keys() {
                check_key_empty(keys, key_value.comment_directives(), l).await;
            }
        }
    }
}

async fn check_key_empty(
    keys: tombi_ast::Keys,
    comment_directives: impl Iterator<Item = tombi_ast::TombiValueCommentDirective>,
    l: &mut crate::Linter<'_>,
) {
    let level = get_comment_directive_content::<KeyFormatRules, KeyCommonExtensibleLintRules>(
        comment_directives,
    )
    .as_ref()
    .and_then(|comment_directive| comment_directive.lint_rules())
    .map(|rules| &rules.value)
    .and_then(|rules| rules.key_empty.as_ref().map(SeverityLevelDefaultWarn::from))
    .unwrap_or_else(|| {
        l.options()
            .rules
            .as_ref()
            .and_then(|rules| rules.key_empty)
            .unwrap_or_default()
    });

    if level == SeverityLevel::Off {
        return;
    }

    for key in keys.keys() {
        if match &key {
            tombi_ast::Key::BareKey(_) => false,
            tombi_ast::Key::BasicString(node) => node.syntax().text() == "\"\"",
            tombi_ast::Key::LiteralString(node) => node.syntax().text() == "''",
        } {
            l.extend_diagnostics(crate::Diagnostic {
                kind: crate::DiagnosticKind::KeyEmpty,
                level: level.into(),
                range: key.range(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_lint;

    test_lint! {
        #[test]
        fn test_warning_empty(
            r#"
            "" = 1
            "#,
        ) -> Err([
            crate::DiagnosticKind::KeyEmpty
        ]);
    }

    test_lint! {
        #[test]
        fn test_empty_key_with_leading_comment_directive(
            r#"
            # tombi: lint.rules.key-empty.disabled = true
            "" = 1
            "#,
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_empty_key_with_trailing_comment_directive(
            r#"
            "" = 1 # tombi: lint.rules.key-empty.disabled = true
            "#,
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_key_value_empty_key(
            r#"
            a."".b = 1
            "#,
        ) -> Err([
            crate::DiagnosticKind::KeyEmpty
        ]);
    }

    test_lint! {
        #[test]
        fn test_key_value_empty_key_with_leading_comment_directive(
            r#"
            # tombi: lint.rules.key-empty.disabled = true
            a."".b = 1
            "#,
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_key_value_empty_key_with_trailing_comment_directive(
            r#"
            a."".b = 1 # tombi: lint.rules.key-empty.disabled = true
            "#,
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_inline_table_empty_key(
            r#"
            a = { "".b = 1 }
            "#,
        ) -> Err([
            crate::DiagnosticKind::KeyEmpty
        ]);
    }

    test_lint! {
        #[test]
        fn test_inline_table_empty_key_with_leading_comment_directive(
            r#"
            a = {
              # tombi: lint.rules.key-empty.disabled = true
              "".b = 1
            }
            "#,
            TomlVersion(TomlVersion::V1_1_0_Preview),
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_inline_table_empty_key_with_trailing_comment_directive(
            r#"
            a = {
              "".b = 1  # tombi: lint.rules.key-empty.disabled = true
            }
            "#,
            TomlVersion(TomlVersion::V1_1_0_Preview),
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_table_empty_key(
            r#"
            [table]
            "" = 1
            "#,
        ) -> Err([
            crate::DiagnosticKind::KeyEmpty
        ]);
    }

    test_lint! {
        #[test]
        fn test_table_empty_key_with_leading_comment_directive(
            r#"
            [table]
            # tombi: lint.rules.key-empty.disabled = true
            "" = 1
            "#,
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_table_empty_key_with_trailing_comment_directive(
            r#"
            [table]
            "" = 1 # tombi: lint.rules.key-empty.disabled = true
            "#,
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_array_of_table_empty_key(
            r#"
            [[table]]
            "" = 1
            "#,
        ) -> Err([
            crate::DiagnosticKind::KeyEmpty
        ]);
    }

    test_lint! {
        #[test]
        fn test_array_of_table_empty_key_with_leading_comment_directive(
            r#"
            [[table]]
            # tombi: lint.rules.key-empty.disabled = true
            "" = 1
            "#,
        ) -> Ok(_);
    }

    test_lint! {
        #[test]
        fn test_array_of_table_empty_key_with_trailing_comment_directive(
            r#"
            [[table]]
            "" = 1 # tombi: lint.rules.key-empty.disabled = true
            "#,
        ) -> Ok(_);
    }
}
