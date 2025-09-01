use crate::Lint;

impl Lint for tombi_ast::ArrayOfTable {
    async fn lint(&self, l: &mut crate::Linter<'_>) {
        for key_value in self.key_values() {
            key_value.lint(l).await;
        }
    }
}
