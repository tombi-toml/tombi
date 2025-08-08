use crate::SyntaxKind;
use super::traits::{AstNode, AstToken, SyntaxNode, SyntaxToken};
use super::tokens::Identifier;

/// Extras list node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtrasList {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for ExtrasList {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::EXTRAS_LIST
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl ExtrasList {
    pub fn extras(&self) -> impl Iterator<Item = String> + '_ {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .filter_map(Identifier::cast)
            .map(|id| id.text().to_string())
    }

    pub fn bracket_start(&self) -> Option<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::BRACKET_START)
    }

    pub fn bracket_end(&self) -> Option<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::BRACKET_END)
    }
}