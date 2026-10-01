use tombi_config::{SeverityLevel, TomlVersion};

use crate::{Diagnostic, DiagnosticKind, Rule};

pub struct InlineTableTomlVersionRule;

impl Rule<tombi_ast_syntax::InlineTable<'_>> for InlineTableTomlVersionRule {
    async fn check(node: &tombi_ast_syntax::InlineTable<'_>, l: &mut crate::Linter<'_>) {
        if l.toml_version() != TomlVersion::V1_0_0 {
            return;
        }
        if node.has_newlines_between_braces() {
            l.extend_diagnostics(Diagnostic {
                kind: DiagnosticKind::InlineTableMustSingleLine,
                level: SeverityLevel::Error,
                span: node.span(),
            });
        }
        if node.has_last_key_value_trailing_comma()
            && let Some(comma_span) = node
                .key_values_with_comma()
                .last()
                .and_then(|(_, comma)| comma.map(|c| c.span()))
        {
            l.extend_diagnostics(Diagnostic {
                kind: DiagnosticKind::ForbiddenInlineTableLastComma,
                level: SeverityLevel::Error,
                span: comma_span,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_lint;

    test_lint! {
        #[test]
        fn inline_table_trailing_comma_v1_0_0(
            "key = { a = 1, b = 2, }",
            TomlVersion::V1_0_0,
        ) -> Err([crate::DiagnosticKind::ForbiddenInlineTableLastComma])
    }

    test_lint! {
        #[test]
        fn inline_table_trailing_comma_v1_1_0_ok(
            "key = { a = 1, b = 2, }",
            TomlVersion::V1_1_0,
        ) -> Ok(_)
    }

    test_lint! {
        #[test]
        fn inline_table_multi_line_v1_0_0(
            r#"
            key = {
                key1 = 1,
                key2 = 2,
            }
            "#,
            TomlVersion::V1_0_0,
        ) -> Err([
            crate::DiagnosticKind::InlineTableMustSingleLine,
            crate::DiagnosticKind::ForbiddenInlineTableLastComma,
        ])
    }

    test_lint! {
        #[test]
        fn inline_table_multi_line_v1_0_0_no_trailing_comma(
            r#"
            json_like = {
                first = "Tom",
                last = "Preston-Werner"
            }
            "#,
            TomlVersion::V1_0_0,
        ) -> Err([crate::DiagnosticKind::InlineTableMustSingleLine])
    }

    test_lint! {
        #[test]
        fn inline_table_multi_line_v1_0_0_two_lines(
            r#"
            t = {a=1,
            b=2}
            "#,
            TomlVersion::V1_0_0,
        ) -> Err([crate::DiagnosticKind::InlineTableMustSingleLine])
    }

    test_lint! {
        #[test]
        fn inline_table_multi_line_v1_1_0_ok(
            r#"
            key = {
                key1 = 1,
                key2 = 2,
            }
            "#,
            TomlVersion::V1_1_0,
        ) -> Ok(_)
    }

    test_lint! {
        #[test]
        fn inline_table_with_multiline_array_value_v1_0_0_ok(
            r#"
            key = { arr = [
                1,
                2,
                3
            ] }
            "#,
            TomlVersion::V1_0_0,
        ) -> Ok(_)
    }

    test_lint! {
        #[test]
        fn inline_table_with_multiline_array_and_other_value_v1_0_0_ok(
            r#"
            key = { arr = [
                1,
                2
            ], b = 2 }
            "#,
            TomlVersion::V1_0_0,
        ) -> Ok(_)
    }
}
