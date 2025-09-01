use crate::Lint;

impl Lint for tombi_ast::KeyValue {
    async fn lint(&self, l: &mut crate::Linter<'_>) {
        if let Some(keys) = self.keys() {
            for key in keys.keys() {
                key.lint(l).await;
            }

            if let Some(value) = self.value() {
                value.lint(l).await;
            }
        }
    }
}
