use crate::SyntaxKind;
use super::traits::{AstToken, SyntaxToken};

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

/// Version string token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionString {
    pub(crate) syntax: SyntaxToken,
}

impl std::fmt::Display for VersionString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.syntax, f)
    }
}

impl AstToken for VersionString {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VERSION_STRING
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