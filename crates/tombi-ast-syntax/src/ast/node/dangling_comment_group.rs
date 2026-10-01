use crate::{AstNode, AstToken, DanglingComment};
use tombi_ast_syntax::{SyntaxKind::DANGLING_COMMENT_GROUP, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DanglingCommentGroup<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}

impl<'t> DanglingCommentGroup<'t> {
    pub fn comments(&self) -> impl Iterator<Item = DanglingComment<'t>> {
        self.syntax()
            .child_elements()
            .filter_map(|el| el.into_token().and_then(DanglingComment::cast))
    }

    pub fn into_comments(self) -> impl Iterator<Item = DanglingComment<'t>> {
        self.syntax()
            .child_elements()
            .filter_map(|el| el.into_token().and_then(DanglingComment::cast))
    }

    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

impl<'t> AstNode<'t> for DanglingCommentGroup<'t> {
    #[inline]
    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool {
        kind == DANGLING_COMMENT_GROUP
    }

    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
