use tombi_ast::DanglingCommentGroupOr;
use tombi_config::SeverityLevel;
use tombi_text::Range;

use crate::{Diagnostic, DiagnosticKind, Rule};

pub struct MissingCommaRule;

impl Rule<tombi_ast::Array> for MissingCommaRule {
    async fn check(node: &tombi_ast::Array, l: &mut crate::Linter<'_>) {
        let mut values_with_comma = vec![];
        for group in node.value_with_comma_groups() {
            if let DanglingCommentGroupOr::ItemGroup(value_group) = group {
                values_with_comma.extend(value_group.values_with_comma());
            }
        }
        let mut values_with_comma = values_with_comma.into_iter().peekable();

        while let Some((value, comma)) = values_with_comma.next() {
            if values_with_comma.peek().is_some() && comma.is_none() {
                l.extend_diagnostics(Diagnostic {
                    kind: DiagnosticKind::MissingArrayComma,
                    level: SeverityLevel::Error,
                    range: Range::at(value.range().end),
                });
            }
        }
    }
}

impl Rule<tombi_ast::InlineTable> for MissingCommaRule {
    async fn check(node: &tombi_ast::InlineTable, l: &mut crate::Linter<'_>) {
        let mut key_values_with_comma = vec![];
        for group in node.key_value_with_comma_groups() {
            if let DanglingCommentGroupOr::ItemGroup(key_value_group) = group {
                key_values_with_comma.extend(key_value_group.key_values_with_comma());
            }
        }

        let mut key_values_with_comma = key_values_with_comma.into_iter().peekable();

        while let Some((key_value, comma)) = key_values_with_comma.next() {
            if key_values_with_comma.peek().is_some() && comma.is_none() {
                l.extend_diagnostics(Diagnostic {
                    kind: DiagnosticKind::MissingInlineTableComma,
                    level: SeverityLevel::Error,
                    range: Range::at(key_value.range().end),
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
        fn array_missing_comma(
            "key = [1 2, 3]"
        ) -> Err([crate::DiagnosticKind::MissingArrayComma])
    }

    test_lint! {
        #[test]
        fn array_last_value_without_comma_ok(
            "key = [1]"
        ) -> Ok(_)
    }

    test_lint! {
        #[test]
        fn inline_table_missing_comma(
            "key = { a = 1 b = 2, c = 3 }"
        ) -> Err([crate::DiagnosticKind::MissingInlineTableComma])
    }

    test_lint! {
        #[test]
        fn inline_table_last_key_value_without_comma_ok(
            "key = { a = 1 }"
        ) -> Ok(_)
    }
}
