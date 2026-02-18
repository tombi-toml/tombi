use crate::{AstChildren, AstNode};

#[inline]
pub fn child<N: AstNode>(parent: &tombi_syntax::SyntaxNode) -> Option<N> {
    parent.children().find_map(N::cast)
}

#[inline]
pub fn children<N: AstNode>(parent: &tombi_syntax::SyntaxNode) -> AstChildren<N> {
    AstChildren::new(parent)
}

#[inline]
pub fn token(
    parent: &tombi_syntax::SyntaxNode,
    kind: tombi_syntax::SyntaxKind,
) -> Option<tombi_syntax::SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|node_or_token| node_or_token.into_token())
        .find(|token| token.kind() == kind)
}

pub fn prev_siblings_nodes<N: AstNode, T: AstNode>(node: &N) -> impl Iterator<Item = T> {
    node.syntax()
        .siblings(tombi_syntax::Direction::Prev)
        .skip(1)
        .filter_map(T::cast)
}

pub fn next_siblings_nodes<N: AstNode, T: AstNode>(node: &N) -> impl Iterator<Item = T> {
    node.syntax()
        .siblings(tombi_syntax::Direction::Next)
        .filter_map(T::cast)
}
