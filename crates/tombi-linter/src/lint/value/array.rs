use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast::Array {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            for value in self.values() {
                value.lint(l).await;
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
            fn test_array_min_values(
                r#"
                array = []
                "#,
                type_test_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::ArrayMinValues {
                    min_values: 2,
                    actual: 0,
                }
            ]);
        }

        test_lint! {
            #[test]
            fn test_array_min_values_with_leading_comment_directive(
                r#"
                # tombi: lint.rules.array-min-values.disabled = true
                array = []
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_min_values_with_trailing_comment_directive(
                r#"
                array = [] # tombi: lint.rules.array-min-values.disabled = true
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_min_values_with_dangling_comment_directive(
                r#"
                array = [
                  # tombi: lint.rules.array-min-values.disabled = true
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_min_values_with_begin_dangling_comment_directive(
                r#"
                array = [
                  # tombi: lint.rules.array-min-values.disabled = true

                  1,
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_min_values_with_end_dangling_comment_directive(
                r#"
                array = [
                  1,

                  # tombi: lint.rules.array-min-values.disabled = true
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_min_values_with_leading_and_end_dangling_comment_directive(
                r#"
                # tombi: lint.rules.array-max-values.disabled = true
                array = [
                  1,

                  # tombi: lint.rules.array-min-values.disabled = true
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_min_values_with_key_leading_and_array_dangling_comment_directive(
                r#"
                # tombi: lint.rules.key-empty.disabled = true
                array = [
                  # tombi: lint.rules.array-min-values.disabled = true
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_array_min_values_with_array_leading_and_key_dangling_comment_directive(
                r#"
                # tombi: lint.rules.array-min-values.disabled = true
                array = [
                    # tombi: lint.rules.key-empty.disabled = true
                ]
                "#,
                type_test_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::KeyNotAllowed {key: "key-empty".to_string()}
            ]);
        }

        test_lint! {
            #[test]
            fn test_nested_array(
                r#"
                array = [[
                    # tombi: lint.rules.array-min-values.disabled = true
                ]]
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
            fn test_nested_array_min_values_with_array_leading_and_key_dangling_comment_directive(
                r#"
                # tombi: lint.rules.array-min-values.disabled = true
                array = [
                    [
                    # tombi: lint.rules.key-empty.disabled = true
                    # tombi: lint.rules.array-min-values.disabled = true
                    ]
                ]
                "#,
                type_test_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::KeyNotAllowed {key: "key-empty".to_string()}
            ]);
        }

        test_lint! {
            #[test]
            fn test_nested_array_integer_min_values_with_array_leading_and_array_dangling_comment_directive(
                r#"
                # tombi: lint.rules.array-min-values.disabled = true
                array = [
                    [
                    # tombi: lint.rules.array-min-values.disabled = true

                    # tombi: lint.rules.key-empty.disabled = true
                    0, # tombi: lint.rules.integer-minimum.disabled = true
                    ]
                ]
                "#,
                type_test_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::KeyNotAllowed {key: "key-empty".to_string()}
            ]);
        }
    }
}
