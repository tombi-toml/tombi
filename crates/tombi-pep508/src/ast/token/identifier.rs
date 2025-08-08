use crate::ast::traits::{AstToken, SyntaxToken};
use crate::SyntaxKind;

/// Identifier token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub(crate) syntax: SyntaxToken,
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.syntax, f)
    }
}

impl AstToken for Identifier {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::IDENTIFIER
    }

    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}
