use tombi_ast::DanglingCommentGroupOr;
use tombi_config::SeverityLevel;

use crate::{Diagnostic, DiagnosticKind, Rule};

pub struct TrailingCommaRule;

fn check_key_value_groups(
    groups: impl Iterator<Item = DanglingCommentGroupOr<tombi_ast::KeyValueGroup>>,
    l: &mut crate::Linter<'_>,
) {
    for group in groups {
        let DanglingCommentGroupOr::ItemGroup(key_value_group) = group else {
            continue;
        };

        for (_, comma) in key_value_group.key_values_with_comma() {
            let Some(comma) = comma else {
                continue;
            };

            if let Some(comma_token) = comma.comma() {
                l.extend_diagnostics(Diagnostic {
                    kind: DiagnosticKind::ForbiddenKeyValueTrailingComma,
                    level: SeverityLevel::Error,
                    range: comma_token.range(),
                });
            }
        }
    }
}

impl Rule<tombi_ast::Root> for TrailingCommaRule {
    async fn check(node: &tombi_ast::Root, l: &mut crate::Linter<'_>) {
        check_key_value_groups(node.key_value_groups(), l);
    }
}

impl Rule<tombi_ast::Table> for TrailingCommaRule {
    async fn check(node: &tombi_ast::Table, l: &mut crate::Linter<'_>) {
        check_key_value_groups(node.key_value_groups(), l);
    }
}

impl Rule<tombi_ast::ArrayOfTable> for TrailingCommaRule {
    async fn check(node: &tombi_ast::ArrayOfTable, l: &mut crate::Linter<'_>) {
        check_key_value_groups(node.key_value_groups(), l);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_lint;

    test_lint! {
        #[test]
        fn root_key_value_trailing_comma(
            r#"
            key1 = 1,
            key2 = 2
            "#
        ) -> Err([crate::DiagnosticKind::ForbiddenKeyValueTrailingComma])
    }

    test_lint! {
        #[test]
        fn table_key_value_trailing_comma(
            r#"
            [package]
            name = "toml-rs",
            version = "0.4.0"
            "#
        ) -> Err([crate::DiagnosticKind::ForbiddenKeyValueTrailingComma])
    }

    test_lint! {
        #[test]
        fn array_of_table_key_value_trailing_comma(
            r#"
            [[package]]
            name = "toml-rs",
            version = "0.4.0"
            "#
        ) -> Err([crate::DiagnosticKind::ForbiddenKeyValueTrailingComma])
    }

    test_lint! {
        #[test]
        fn key_value_without_trailing_comma_ok(
            r#"
            [package]
            name = "toml-rs"
            version = "0.4.0"
            "#
        ) -> Ok(_)
    }
}
