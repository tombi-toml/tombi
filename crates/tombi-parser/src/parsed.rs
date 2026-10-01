use tombi_ast_syntax::{AstNode, SyntaxTree};

/// The result of parsing a source text.
///
/// It owns the syntax tape, which borrows the source for `'src`.
/// The root and every node reached from it borrow the result.
#[derive(Debug)]
pub struct ParseResult<'src> {
    tree: SyntaxTree<'src>,
    pub errors: Vec<crate::Error>,
    pub line_ending: tombi_text::LineEnding,
}

impl<'src> ParseResult<'src> {
    pub(crate) fn new(
        tree: SyntaxTree<'src>,
        errors: Vec<crate::Error>,
        line_ending: tombi_text::LineEnding,
    ) -> Self {
        Self {
            tree,
            errors,
            line_ending,
        }
    }

    /// The line index built while parsing, to convert spans of the source into ranges.
    #[inline]
    pub fn line_index(&self) -> &tombi_text::LineIndex<'src> {
        self.tree.line_index()
    }

    /// The source text that was parsed.
    #[inline]
    pub fn source(&self) -> &'src str {
        self.tree.line_index().text()
    }

    pub fn root(&self) -> tombi_ast_syntax::Root<'_> {
        tombi_ast_syntax::Root::cast(self.tree.root())
            .expect("a TOML parse always produces a root node")
    }

    /// The root, or the errors if the source has any.
    #[inline]
    pub fn try_root(&self) -> Result<tombi_ast_syntax::Root<'_>, &[crate::Error]> {
        self.errors
            .is_empty()
            .then(|| self.root())
            .ok_or(&self.errors[..])
    }
}
