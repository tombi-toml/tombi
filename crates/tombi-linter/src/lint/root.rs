use crate::{Lint, Rule};

impl Lint for tombi_ast::Root {
    async fn lint(&self, l: &mut crate::Linter<'_>) {
        // Apply root-level rules
        crate::rule::DottedKeysOutOfOrderRule::check(self, l).await;
        crate::rule::TablesOutOfOrderRule::check(self, l).await;

        for item in self.items() {
            item.lint(l).await;
        }
    }
}

impl Lint for tombi_ast::RootItem {
    async fn lint(&self, l: &mut crate::Linter<'_>) {
        match self {
            Self::Table(table) => table.lint(l).await,
            Self::ArrayOfTable(array_of_table) => array_of_table.lint(l).await,
            Self::KeyValue(key_value) => key_value.lint(l).await,
        }
    }
}
