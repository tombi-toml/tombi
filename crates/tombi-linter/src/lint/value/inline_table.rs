use tombi_future::Boxable;

use crate::{Lint, rule::Rule};

impl Lint for tombi_ast::InlineTable {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            crate::rule::MissingCommaRule::check(self, l).await;
            crate::rule::DottedKeysOutOfOrderRule::check(self, l).await;
            crate::rule::InlineTableTomlVersionRule::check(self, l).await;

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
            fn test_inline_table_key_empty(
                r#"
                #:tombi schema.strict = false

                table = { "" = 1, boolean = true }
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Err([tombi_validator::DiagnosticKind::KeyEmpty])
        }

        test_lint! {
            #[test]
            fn test_inline_table_key_empty_with_leading_comment_directive(
                r#"
                #:tombi schema.strict = false

                table = {
                    # tombi: lint.rules.key-empty.disabled = true
                    # tombi: lint.rules.table-min-keys.disabled = true
                    "" = 1,
                    boolean = true,
                }
                "#,
                SchemaPath(type_test_schema_path()),
                TomlVersion::V1_1_0,
            ) -> Ok(_)
        }

        test_lint! {
            #[test]
            fn test_inline_table_key_empty_with_trailing_comment_directive(
                r#"
                #:tombi schema.strict = false

                table = {
                    "" = 1, # tombi: lint.rules.key-empty.disabled = true
                    boolean = true,
                }
                "#,
                SchemaPath(type_test_schema_path()),
                TomlVersion::V1_1_0,
            ) -> Err([tombi_validator::DiagnosticKind::KeyEmpty])
        }

        test_lint! {
            #[test]
            fn test_inline_table_key_empty_with_dangling_comment_directive(
                r#"
                #:tombi schema.strict = false

                table = {
                    "" = 1,
                    boolean = true,
                    # tombi: lint.rules.key-empty.disabled = true
                }
                "#,
                SchemaPath(type_test_schema_path()),
                TomlVersion::V1_1_0,
            ) -> Err([
                tombi_validator::DiagnosticKind::KeyNotAllowed { key: "key-empty".to_string() },
                tombi_validator::DiagnosticKind::KeyEmpty,
            ])
        }
    }
}
