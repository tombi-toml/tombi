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
