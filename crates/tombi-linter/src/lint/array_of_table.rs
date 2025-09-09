use tombi_future::Boxable;

use crate::{rule::Rule, Lint};

impl Lint for tombi_ast::ArrayOfTable {
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
            fn test_array_of_table_min_values(
                r#"
                [[array]]
                boolean = true
                integer = 1
                "#,
                type_test_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::ArrayMinValues {
                    min_values: 2,
                    actual: 1,
                }
            ]);
        }

        test_lint! {
            #[test]
            fn test_array_of_table_min_values_with_leading_comment_directive(
                r#"
                # tombi: lint.rules.array-min-values.disabled = true
                [[array]]
                boolean = true
                integer = 1
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_of_table_min_values_with_header_trailing_comment_directive(
                r#"
                [[array]] # tombi: lint.rules.array-min-values.disabled = true
                boolean = true
                integer = 1
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_of_table_min_values_with_dangling_comment_directive(
                r#"
                [[array]]
                # tombi: lint.rules.array-min-values.disabled = true
                boolean = true
                integer = 1
                "#,
                type_test_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::KeyNotAllowed {
                    key: "array-min-values".to_string()
                },
                tombi_validator::DiagnosticKind::ArrayMinValues {
                    min_values: 2,
                    actual: 1,
                }
            ]);
        }

        test_lint! {
            #[test]
            fn test_array_of_table_empty_key_min_values_with_comment_directives(
                r#"
                #:tombi schema.strict = false

                # tombi: lint.rules.array-min-values.disabled = true
                [[array]]
                boolean = true
                integer = 1

                # tombi: lint.rules.key-empty.disabled = true
                [array.""]
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_of_table_empty_key_min_values_with_comment_directives2(
                r#"
                #:tombi schema.strict = false

                # tombi: lint.rules.array-min-values.disabled = true
                [[array]]
                boolean = true
                integer = 1

                [array.""]
                # tombi: lint.rules.key-empty.disabled = true
                "#,
                type_test_schema_path(),
            ) -> Err([
                crate::DiagnosticKind::KeyEmpty
            ]);
        }
    }
}
