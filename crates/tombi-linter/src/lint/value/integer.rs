use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast::IntegerBin {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast::IntegerOct {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast::IntegerDec {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast::IntegerHex {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}
