use tombi_ast::AstNode;
use tombi_comment_directive::value::KeyCommonExtensibleRules;
use tombi_config::SeverityLevel;
use tombi_validator::comment_directive::get_tombi_value_rules_and_diagnostics;

use crate::Rule;
use itertools::Itertools;

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
    let level = get_tombi_value_rules_and_diagnostics::<KeyCommonExtensibleRules>(
        &comment_directives.collect_vec(),
    )
    .await
    .0
    .as_ref()
    .map(|rules| &rules.value)
    .and_then(|rules| rules.key_empty)
    .unwrap_or_else(|| {
        l.options()
            .rules
            .as_ref()
            .and_then(|rules| rules.key_empty)
            .unwrap_or_default()
    });

    for key in keys.keys() {
        if match &key {
            tombi_ast::Key::BareKey(_) => false,
            tombi_ast::Key::BasicString(node) => node.syntax().text() == "\"\"",
            tombi_ast::Key::LiteralString(node) => node.syntax().text() == "''",
        } {
            if level == SeverityLevel::Off {
                return;
            }

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
    use std::vec;

    use tombi_diagnostic::SetDiagnostics;

    #[tokio::test]
    async fn test_key_empty() {
        let diagnostics = crate::Linter::new(
            tombi_config::TomlVersion::default(),
            &crate::LintOptions::default(),
            None,
            &tombi_schema_store::SchemaStore::new(),
        )
        .lint("'' = 1")
        .await
        .unwrap_err();

        let mut expected = vec![];
        crate::Diagnostic {
            kind: crate::DiagnosticKind::KeyEmpty,
            level: tombi_config::SeverityLevel::Warn,
            range: tombi_text::Range::new((0, 0).into(), (0, 2).into()),
        }
        .set_diagnostics(&mut expected);

        assert_eq!(diagnostics, expected);
    }
}
