use crate::AstNode;

#[inline]
pub fn child<'t, N: AstNode<'t>>(parent: &tombi_ast_syntax::SyntaxNode<'t>) -> Option<N> {
    parent.child_nodes().find_map(N::cast)
}

#[inline]
pub fn token<'t>(
    parent: &tombi_ast_syntax::SyntaxNode<'t>,
    kind: tombi_ast_syntax::SyntaxKind,
) -> Option<tombi_ast_syntax::SyntaxToken<'t>> {
    parent
        .child_elements()
        .filter_map(|node_or_token| node_or_token.into_token())
        .find(|token| token.kind() == kind)
}

pub fn prev_siblings_nodes<'t, N: AstNode<'t>, T: AstNode<'t>>(
    node: &N,
) -> impl Iterator<Item = T> + use<'t, N, T> {
    node.syntax()
        .siblings(tombi_ast_syntax::Direction::Prev)
        .skip(1)
        .filter_map(T::cast)
}

pub fn next_siblings_nodes<'t, N: AstNode<'t>, T: AstNode<'t>>(
    node: &N,
) -> impl Iterator<Item = T> + use<'t, N, T> {
    node.syntax()
        .siblings(tombi_ast_syntax::Direction::Next)
        .filter_map(T::cast)
}
