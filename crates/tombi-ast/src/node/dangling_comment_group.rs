use crate::{AstNode, AstToken, DanglingComment};
use tombi_syntax::{SyntaxKind::DANGLING_COMMENT_GROUP, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DanglingCommentGroup {
    pub(crate) syntax: SyntaxNode,
}

impl DanglingCommentGroup {
    pub fn comments(&self) -> impl Iterator<Item = DanglingComment> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|el| el.into_token().and_then(DanglingComment::cast))
    }

    pub fn into_comments(self) -> impl Iterator<Item = DanglingComment> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|el| el.into_token().and_then(DanglingComment::cast))
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

impl AstNode for DanglingCommentGroup {
    #[inline]
    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool {
        kind == DANGLING_COMMENT_GROUP
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
