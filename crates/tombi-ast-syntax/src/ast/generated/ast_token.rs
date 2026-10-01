//! Generated file, do not edit by hand, see `xtask/src/codegen`

use crate::AstToken;
use tombi_ast_syntax::{SyntaxKind, SyntaxToken};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct Comment<'t> {
    pub(crate) syntax: SyntaxToken<'t>,
}
impl std::fmt::Display for Comment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.syntax, f)
    }
}
impl<'t> AstToken<'t> for Comment<'t> {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::COMMENT
    }
    fn cast(syntax: SyntaxToken<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken<'t> {
        &self.syntax
    }
}
