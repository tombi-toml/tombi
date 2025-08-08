use crate::SyntaxKind;
use crate::ast::traits::{AstNode, SyntaxNode};
use super::package_name::PackageName;
use super::extras_list::ExtrasList;
use super::version_spec::VersionSpecNode;
use super::url_spec::UrlSpec;
use super::marker_expr::MarkerExpr;

/// A PEP 508 requirement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Requirement {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for Requirement {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::REQUIREMENT
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

impl Requirement {
    pub fn package_name(&self) -> Option<PackageName> {
        self.syntax.children().find_map(PackageName::cast)
    }

    pub fn extras_list(&self) -> Option<ExtrasList> {
        self.syntax.children().find_map(ExtrasList::cast)
    }

    pub fn version_spec(&self) -> Option<VersionSpecNode> {
        self.syntax.children().find_map(VersionSpecNode::cast)
    }

    pub fn url_spec(&self) -> Option<UrlSpec> {
        self.syntax.children().find_map(UrlSpec::cast)
    }

    pub fn marker(&self) -> Option<MarkerExpr> {
        self.syntax.children().find_map(MarkerExpr::cast)
    }
}