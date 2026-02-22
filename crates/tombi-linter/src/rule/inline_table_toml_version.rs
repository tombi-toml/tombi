use tombi_config::{SeverityLevel, TomlVersion};

use crate::{Diagnostic, DiagnosticKind, Rule};

pub struct InlineTableTomlVersionRule;

impl Rule<tombi_ast::InlineTable> for InlineTableTomlVersionRule {
    async fn check(node: &tombi_ast::InlineTable, l: &mut crate::Linter<'_>) {
        if l.toml_version() != TomlVersion::V1_0_0 {
            return;
        }
        let brace_start = match node.brace_start() {
            Some(t) => t,
            None => return,
        };
        let brace_end = match node.brace_end() {
            Some(t) => t,
            None => return,
        };
        let table_span = brace_end.range().start.line - brace_start.range().start.line;
        let key_value_lines: u32 = node
            .key_values()
            .map(|kv| kv.range().end.line - kv.range().start.line)
            .sum();
        if table_span != key_value_lines {
            l.extend_diagnostics(Diagnostic {
                kind: DiagnosticKind::InlineTableMustSingleLine,
                level: SeverityLevel::Error,
                range: node.range(),
            });
        }
        if node.has_last_key_value_trailing_comma() {
            if let Some(comma_range) = node
                .key_values_with_comma()
                .last()
                .and_then(|(_, comma)| comma.map(|c| c.range()))
            {
                l.extend_diagnostics(Diagnostic {
                    kind: DiagnosticKind::ForbiddenInlineTableLastComma,
                    level: SeverityLevel::Error,
                    range: comma_range,
                });
            }
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
}
