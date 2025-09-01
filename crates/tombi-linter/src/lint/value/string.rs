use crate::Lint;

impl Lint for tombi_ast::BasicString {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::LiteralString {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::MultiLineBasicString {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::MultiLineLiteralString {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}
