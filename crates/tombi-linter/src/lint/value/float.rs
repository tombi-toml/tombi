use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast::Float {
    fn lint<'a: 'b, 'b>(
        &'a self,
        _l: &'a mut crate::Linter<'_>,
    ) -> tombi_future::BoxFuture<'b, ()> {
        async move {}.boxed()
    }
}
