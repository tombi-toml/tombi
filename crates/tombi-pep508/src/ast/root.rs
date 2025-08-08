use crate::SyntaxKind;
use super::traits::{AstNode, SyntaxNode};
use super::requirement::Requirement;

/// Root node of a PEP 508 requirement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Root {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for Root {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ROOT
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

impl Root {
    pub fn requirement(&self) -> Option<Requirement> {
        self.syntax.children().find_map(Requirement::cast)
    }
}