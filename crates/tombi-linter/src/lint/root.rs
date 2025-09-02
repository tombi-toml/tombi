use tombi_future::Boxable;

use crate::{Lint, Rule};

impl Lint for tombi_ast::Root {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            crate::rule::KeyEmptyRule::check(self, l).await;
            crate::rule::DottedKeysOutOfOrderRule::check(self, l).await;
            crate::rule::TablesOutOfOrderRule::check(self, l).await;
            for item in self.items() {
                item.lint(l).await;
            }
        }
        .boxed()
    }
}

impl Lint for tombi_ast::RootItem {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            match self {
                Self::Table(table) => table.lint(l).await,
                Self::ArrayOfTable(array_of_table) => array_of_table.lint(l).await,
                Self::KeyValue(key_value) => key_value.lint(l).await,
            }
        }
        .boxed()
    }
}
