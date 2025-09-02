use tombi_future::Boxable;

use crate::{rule::Rule, Lint};

impl Lint for tombi_ast::InlineTable {
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
