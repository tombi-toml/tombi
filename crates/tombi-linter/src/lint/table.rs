use tombi_future::Boxable;

use crate::{rule::Rule, Lint};

impl Lint for tombi_ast::Table {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            crate::rule::KeyEmptyRule::check(self, l).await;
            crate::rule::DottedKeysOutOfOrderRule::check(self, l).await;

            for key_value in self.key_values() {
                key_value.lint(l).await;
            }
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_lint;

    mod type_test {
        use tombi_test_lib::type_test_schema_path;

        use super::*;

        test_lint! {
            #[test]
            fn test_table_min_keys(
                r#"
                [table]
                "#,
                type_test_schema_path(),
            ) -> Err([tombi_validator::DiagnosticKind::TableMinKeys {
                min_keys: 1,
                actual: 0,
            }]);
        }
    }
}
