use crate::SyntaxKind;
use super::traits::{AstNode, SyntaxNode};

/// Marker expression node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarkerExpr {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for MarkerExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::MARKER_EXPR
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

impl MarkerExpr {
    pub fn expression(&self) -> String {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .map(|t| {
                if t.kind() == SyntaxKind::WHITESPACE {
                    " ".to_string()
                } else {
                    t.text().to_string()
                }
            })
            .collect::<String>()
            .trim()
            .to_string()
    }
}