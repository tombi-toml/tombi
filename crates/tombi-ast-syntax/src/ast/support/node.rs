use crate::AstNode;

#[inline]
pub fn child<N: AstNode>(parent: &tombi_ast_syntax::SyntaxNode) -> Option<N> {
    parent.child_nodes().find_map(N::cast)
}

#[inline]
pub fn token(
    parent: &tombi_ast_syntax::SyntaxNode,
    kind: tombi_ast_syntax::SyntaxKind,
) -> Option<tombi_ast_syntax::SyntaxToken> {
    parent
        .child_elements()
        .filter_map(|node_or_token| node_or_token.into_token())
        .find(|token| token.kind() == kind)
}

pub fn prev_siblings_nodes<N: AstNode, T: AstNode>(node: &N) -> impl Iterator<Item = T> {
    node.syntax()
        .siblings(tombi_ast_syntax::Direction::Prev)
        .skip(1)
        .filter_map(T::cast)
}

pub fn next_siblings_nodes<N: AstNode, T: AstNode>(node: &N) -> impl Iterator<Item = T> {
    node.syntax()
        .siblings(tombi_ast_syntax::Direction::Next)
        .filter_map(T::cast)
}
