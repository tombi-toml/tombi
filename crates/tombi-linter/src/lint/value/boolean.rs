use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast::Boolean {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_lint;

    mod non_schema {

        use super::*;

        test_lint! {
            #[test]
            fn test_key_eq_string(
                r#"
                key = "value"  # tombi: lint.rules.string-pattern.disabled = true
                "#
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_key1_key2_eq_string(
                r#"
                key1.key2 = "value"  # tombi: lint.rules.string-pattern.disabled = true
                "#
            ) -> Ok(_);
        }
    }
}
