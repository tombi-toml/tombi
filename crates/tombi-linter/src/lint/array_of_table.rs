use crate::{rule::Rule, Lint};

impl Lint for tombi_ast::ArrayOfTable {
    async fn lint(&self, l: &mut crate::Linter<'_>) {
        crate::rule::DottedKeysOutOfOrderRule::check(self, l).await;

        for key_value in self.key_values() {
            key_value.lint(l).await;
        }
    }
}
