use crate::Lint;

impl Lint for tombi_ast::IntegerBin {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::IntegerOct {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::IntegerDec {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::IntegerHex {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}
