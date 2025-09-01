use crate::{rule::KeyEmptyRule, Lint, Rule};

impl Lint for tombi_ast::Key {
    async fn lint(&self, l: &mut crate::Linter<'_>) {
        KeyEmptyRule::check(self, l).await;
    }
}
