use tombi_ast_syntax::{AstNode, SyntaxNode};

#[derive(Debug)]
pub struct ParseResult {
    syntax: SyntaxNode,
    pub errors: Vec<crate::Error>,
    pub line_ending: tombi_text::LineEnding,
}

impl ParseResult {
    pub(crate) fn new(
        syntax: SyntaxNode,
        errors: Vec<crate::Error>,
        line_ending: tombi_text::LineEnding,
    ) -> Self {
        Self {
            syntax,
            errors,
            line_ending,
        }
    }

    pub fn root(&self) -> tombi_ast_syntax::Root {
        tombi_ast_syntax::Root::cast(self.syntax.clone())
            .expect("a TOML parse always produces a root node")
    }

    pub fn into_root(self) -> tombi_ast_syntax::Root {
        tombi_ast_syntax::Root::cast(self.syntax).expect("a TOML parse always produces a root node")
    }

    pub fn into_root_and_errors(self) -> (tombi_ast_syntax::Root, Vec<crate::Error>) {
        let Self { syntax, errors, .. } = self;
        let root =
            tombi_ast_syntax::Root::cast(syntax).expect("a TOML parse always produces a root node");
        (root, errors)
    }

    #[inline]
    pub fn try_into_root(self) -> Result<tombi_ast_syntax::Root, Vec<crate::Error>> {
        let (root, errors) = self.into_root_and_errors();
        errors.is_empty().then_some(root).ok_or(errors)
    }
}
