use crate::Lint;

impl Lint for tombi_ast::OffsetDateTime {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::LocalDateTime {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::LocalDate {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}

impl Lint for tombi_ast::LocalTime {
    async fn lint(&self, _l: &mut crate::Linter<'_>) {}
}
