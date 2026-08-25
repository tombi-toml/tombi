use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast_syntax::IntegerBin {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast_syntax::IntegerOct {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast_syntax::IntegerDec {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}

impl Lint for tombi_ast_syntax::IntegerHex {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}
