pub mod algo;
mod comment_directive;
mod generated;
mod impls;
mod node;
pub mod support;

use std::{fmt::Debug, marker::PhantomData};

pub use comment_directive::*;
pub use generated::*;
pub use node::*;

pub trait AstNode
where
    Self: Debug,
{
    fn leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        support::node::leading_comments(self.syntax().children_with_tokens())
    }

    fn trailing_comment(&self) -> Option<crate::TrailingComment> {
        self.syntax()
            .last_token()
            .and_then(crate::Comment::cast)
            .map(Into::into)
    }

    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: tombi_syntax::SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &tombi_syntax::SyntaxNode;

    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(token: tombi_syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: tombi_syntax::SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &tombi_syntax::SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: tombi_syntax::SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &tombi_syntax::SyntaxNode) -> Self {
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
