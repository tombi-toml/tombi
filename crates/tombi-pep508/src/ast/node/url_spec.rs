use crate::SyntaxKind;
use crate::ast::traits::{AstNode, SyntaxNode};

/// URL specification node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UrlSpec {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for UrlSpec {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::URL_SPEC
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

impl UrlSpec {
    pub fn url(&self) -> String {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .filter(|t| !t.kind().is_trivia())
            .map(|t| t.text().to_string())
            .collect()
    }
}