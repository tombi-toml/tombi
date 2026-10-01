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
