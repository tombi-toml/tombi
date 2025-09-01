use crate::Lint;

impl Lint for tombi_ast::Array {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}
