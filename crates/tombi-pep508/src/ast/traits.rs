use crate::SyntaxKind;
use std::marker::PhantomData;

// Re-export AST types
pub use crate::language::Pep508Language;
pub type SyntaxNode = tombi_rg_tree::RedNode<Pep508Language>;
pub type SyntaxToken = tombi_rg_tree::RedToken<Pep508Language>;
pub type SyntaxElement = tombi_rg_tree::RedElement<Pep508Language>;
pub type SyntaxNodePtr = tombi_rg_tree::RedNodePtr<Pep508Language>;
pub type SyntaxNodeChildren = tombi_rg_tree::RedNodeChildren<Pep508Language>;
pub type SyntaxElementChildren = tombi_rg_tree::RedElementChildren<Pep508Language>;
pub type PreorderWithTokens = tombi_rg_tree::RedPreorderWithTokens<Pep508Language>;

pub trait AstNode: std::fmt::Debug {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;

    fn range(&self) -> tombi_text::Range {
        self.syntax().range()
    }

    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }
}

pub trait AstToken {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    pub fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}