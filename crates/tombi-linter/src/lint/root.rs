use crate::{Lint, Rule};

impl Lint for tombi_ast::Root {
    fn lint(&self, l: &mut crate::Linter) {
        // Apply root-level rules
        crate::rule::DottedKeysOutOfOrderRule::check(self, l);
        crate::rule::TablesOutOfOrderRule::check(self, l);

        for item in self.items() {
            item.lint(l);
        }
    }
}

impl Lint for tombi_ast::RootItem {
    fn lint(&self, l: &mut crate::Linter) {
        match self {
            Self::Table(table) => table.lint(l),
            Self::ArrayOfTable(array_of_table) => array_of_table.lint(l),
            Self::KeyValue(key_value) => key_value.lint(l),
        }
    }
}
