use tombi_ast::AstNode;

use crate::Rule;

pub struct KeyEmptyRule;

impl Rule<tombi_ast::Key> for KeyEmptyRule {
    fn check(node: &tombi_ast::Key, l: &mut crate::Linter) {
        if match node {
            tombi_ast::Key::BareKey(_) => false,
            tombi_ast::Key::BasicString(node) => node.syntax().text() == "\"\"",
            tombi_ast::Key::LiteralString(node) => node.syntax().text() == "''",
        } {
            l.extend_diagnostics(crate::Diagnostic {
                kind: crate::DiagnosticKind::KeyEmpty,
                level: l
                    .options()
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.key_empty)
                    .unwrap_or_default()
                    .into(),
                range: node.syntax().range(),
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
