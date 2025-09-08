use tombi_future::Boxable;

use crate::{Lint, Rule};

impl Lint for tombi_ast::Root {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            crate::rule::KeyEmptyRule::check(self, l).await;
            crate::rule::DottedKeysOutOfOrderRule::check(self, l).await;
            crate::rule::TablesOutOfOrderRule::check(self, l).await;
            for item in self.items() {
                item.lint(l).await;
            }
        }
        .boxed()
    }
}

impl Lint for tombi_ast::RootItem {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            match self {
                Self::Table(table) => table.lint(l).await,
                Self::ArrayOfTable(array_of_table) => array_of_table.lint(l).await,
                Self::KeyValue(key_value) => key_value.lint(l).await,
            }
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    mod type_test_schema {
        use tombi_test_lib::type_test_schema_path;

        use crate::test_lint;

        test_lint! {
            #[test]
            fn test_root_type_mismatch(
                r#"
                integer = "1"
                "#,
                type_test_schema_path(),
            ) -> Err([tombi_validator::DiagnosticKind::TypeMismatch {
                expected: tombi_schema_store::ValueType::Integer,
                actual: tombi_document_tree::ValueType::String,
            }]);
        }

        test_lint! {
            #[test]
            fn test_root_type_mismatch_with_leading_comment_directive(
                r#"
                # tombi: lint.rules.type-mismatch.disabled = true
                integer = "1"
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_root_type_mismatch_with_trailing_comment_directive(
                r#"
                integer = "1"  # tombi: lint.rules.type-mismatch.disabled = true
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_root_key_not_allowed_with_leading_comment_directive(
                r#"
                # tombi: lint.rules.key-not-allowed.disabled = true
                unknown = "value"
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_root_key_not_allowed_with_trailing_comment_directive(
                r#"
                unknown = "value"  # tombi: lint.rules.key-not-allowed.disabled = true
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_root_table_unknown_key_not_allowed_with_leading_comment_directive(
                r#"
                # tombi: lint.rules.key-not-allowed.disabled = true
                table.unknown = "value"
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_root_table_unknown_key_not_allowed_with_trailing_comment_directive(
                r#"
                table.unknown = "value"  # tombi: lint.rules.key-not-allowed.disabled = true
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }
    }
}
