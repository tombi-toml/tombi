use crate::SyntaxKind;
use super::traits::{AstNode, SyntaxNode, AstChildren};
use super::data::VersionOperator;

/// Version specification node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionSpecNode {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for VersionSpecNode {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VERSION_SPEC
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

impl VersionSpecNode {
    pub fn clauses(&self) -> AstChildren<VersionClauseNode> {
        AstChildren::new(&self.syntax)
    }
}

/// Version clause node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionClauseNode {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for VersionClauseNode {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VERSION_CLAUSE
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

impl VersionClauseNode {
    pub fn operator(&self) -> Option<VersionOperator> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind().is_version_operator())
            .map(|t| VersionOperator::from(t.kind()))
    }

    pub fn version(&self) -> Option<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::VERSION_STRING)
            .map(|t| t.text().to_string())
    }
}