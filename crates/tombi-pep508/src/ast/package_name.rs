use crate::SyntaxKind;
use super::traits::{AstNode, AstToken, SyntaxNode};
use super::tokens::Identifier;

/// Package name node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageName {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for PackageName {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::PACKAGE_NAME
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

impl PackageName {
    pub fn identifier(&self) -> Option<Identifier> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find_map(Identifier::cast)
    }

    pub fn name(&self) -> Option<String> {
        self.identifier().map(|id| id.text().to_string())
    }
}