use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast::KeyValue {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            if let Some(value) = self.value() {
                value.lint(l).await;
            }
        }
        .boxed()
    }
}
