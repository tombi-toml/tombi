use crate::{AstNode, AstToken, Comment};
use tombi_syntax::{SyntaxKind::DANGLING_COMMENTS, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DanglingComments {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for DanglingComments {
    #[inline]
    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool {
        kind == DANGLING_COMMENTS
    }

    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl DanglingComments {
    /// Returns all comments (flattened)
    pub fn comments(&self) -> impl Iterator<Item = Comment> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|el| el.into_token().and_then(Comment::cast))
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}
