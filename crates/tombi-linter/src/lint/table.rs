use tombi_future::Boxable;

use crate::{Lint, rule::Rule};

impl Lint for tombi_ast::Table {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            crate::rule::DottedKeysOutOfOrderRule::check(self, l).await;
            crate::rule::TrailingCommaRule::check(self, l).await;

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
                SchemaPath(type_test_schema_path()),
            ) -> Err([tombi_validator::DiagnosticKind::TableMinKeys {
                min_keys: 2,
                actual: 0,
            }])
        }

        test_lint! {
            #[test]
            fn test_table_min_keys_with_leading_comment_directive(
                r#"
                # tombi: lint.rules.table-min-keys.disabled = true
                [table]
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Ok(_)
        }

        test_lint! {
            #[test]
            fn test_table_unknown_min_keys(
                r#"
                #:tombi schema.strict = false

                # tombi: lint.rules.table-min-keys.disabled = true
                [table.""]
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Err([tombi_validator::DiagnosticKind::KeyEmpty])
        }

        test_lint! {
            #[test]
            fn test_table_unknown_min_keys_with_leading_comment_directive(
                r#"
                #:tombi schema.strict = false

                # tombi: lint.rules.key-empty.disabled = true
                # tombi: lint.rules.table-min-keys.disabled = true
                [table.""]
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Ok(_)
        }

        test_lint! {
            #[test]
            fn test_table_unknown_min_keys_with_trailing_comment_directive(
                r#"
                #:tombi schema.strict = false

                # tombi: lint.rules.table-min-keys.disabled = true
                [table.""] # tombi: lint.rules.key-empty.disabled = true
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Ok(_)
        }

        test_lint! {
            #[test]
            fn test_table_unknown_min_keys_with_dangling_comment_directive(
                r#"
                #:tombi schema.strict = false

                # tombi: lint.rules.table-min-keys.disabled = true
                [table.""]
                # tombi: lint.rules.key-empty.disabled = true
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Err([tombi_validator::DiagnosticKind::KeyEmpty])
        }

        test_lint! {
            #[test]
            fn test_table_allows_empty_key_with_property_names_min_length_0(
                r#"
                [table-allows-empty-key]
                "" = 1
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Ok(_)
        }
    }

    mod if_then_else_test {
        use tombi_test_lib::if_then_else_test_schema_path;

        use super::*;

        // if/then/else: `if` matches (country = "USA"), `then` schema applied → valid zip code
        test_lint! {
            #[test]
            fn test_if_then_else_then_branch_valid(
                r#"
                [conditional-table]
                country = "USA"
                postal-code = "12345"
                "#,
                SchemaPath(if_then_else_test_schema_path()),
            ) -> Ok(_)
        }

        // if/then/else: `if` matches (country = "USA"), `then` schema applied → invalid zip code
        test_lint! {
            #[test]
            fn test_if_then_else_then_branch_invalid(
                r#"
                [conditional-table]
                country = "USA"
                postal-code = "ABC"
                "#,
                SchemaPath(if_then_else_test_schema_path()),
            ) -> Err([tombi_validator::DiagnosticKind::StringPattern {
                pattern: "^[0-9]{5}$".to_string(),
                actual: "\"ABC\"".to_string(),
            }])
        }

        // if/then/else: `if` does not match (country = "Canada"), `else` schema applied → valid postal code
        test_lint! {
            #[test]
            fn test_if_then_else_else_branch_valid(
                r#"
                [conditional-table]
                country = "Canada"
                postal-code = "A1B 2C3"
                "#,
                SchemaPath(if_then_else_test_schema_path()),
            ) -> Ok(_)
        }

        // if/then/else: `if` does not match (country = "Canada"), `else` schema applied → invalid postal code
        test_lint! {
            #[test]
            fn test_if_then_else_else_branch_invalid(
                r#"
                [conditional-table]
                country = "Canada"
                postal-code = "12345"
                "#,
                SchemaPath(if_then_else_test_schema_path()),
            ) -> Err([tombi_validator::DiagnosticKind::StringPattern {
                pattern: "^[A-Z][0-9][A-Z] [0-9][A-Z][0-9]$".to_string(),
                actual: "\"12345\"".to_string(),
            }])
        }

        // if only (no then/else): should produce no errors regardless
        test_lint! {
            #[test]
            fn test_if_only_no_error(
                r#"
                [conditional-table-if-only]
                country = "USA"
                postal-code = "anything"
                "#,
                SchemaPath(if_then_else_test_schema_path()),
            ) -> Ok(_)
        }

        // if + then only (no else): `if` matches → `then` applied
        test_lint! {
            #[test]
            fn test_if_then_only_then_branch_valid(
                r#"
                [conditional-table-if-then-only]
                country = "USA"
                postal-code = "12345"
                "#,
                SchemaPath(if_then_else_test_schema_path()),
            ) -> Ok(_)
        }

        // if + then only (no else): `if` does not match → no else, so no error
        test_lint! {
            #[test]
            fn test_if_then_only_else_branch_no_error(
                r#"
                [conditional-table-if-then-only]
                country = "Canada"
                postal-code = "anything"
                "#,
                SchemaPath(if_then_else_test_schema_path()),
            ) -> Ok(_)
        }
    }
}
