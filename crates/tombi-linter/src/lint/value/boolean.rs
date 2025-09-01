use crate::Lint;

impl Lint for tombi_ast::Boolean {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}
