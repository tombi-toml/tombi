use tombi_ast_syntax::Boolean;

use super::LiteralNode;

impl<'t> LiteralNode<'t> for Boolean<'t> {
    fn token(&self) -> Option<tombi_ast_syntax::SyntaxToken<'t>> {
        self.token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Formatter, test_format};

    test_format! {
        #[tokio::test]
        async fn boolean_true(r#"boolean = true"#) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn boolean_false(r#"boolean = false"#) -> Ok(source)
    }
}
