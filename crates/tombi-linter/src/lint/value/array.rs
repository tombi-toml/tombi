use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast::Array {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            for value in self.values() {
                value.lint(l).await;
            }
        }
        .boxed()
    }
}
