use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast::OffsetDateTime {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast::LocalDateTime {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast::LocalDate {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast::LocalTime {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}
