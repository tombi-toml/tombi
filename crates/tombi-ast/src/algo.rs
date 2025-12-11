use itertools::Itertools;
use tombi_syntax::SyntaxNode;

use crate::AstNode;

pub fn find_node_at_offset<N: AstNode>(
    syntax: &SyntaxNode,
    offset: tombi_text::Offset,
) -> Option<N> {
    ancestors_at_offset(syntax, offset).find_map(N::cast)
}

pub fn find_node_at_position<N: AstNode>(
    syntax: &SyntaxNode,
    position: tombi_text::Position,
) -> Option<N> {
    ancestors_at_position(syntax, position).find_map(N::cast)
}

pub fn ancestors_at_offset(
    node: &SyntaxNode,
    offset: tombi_text::Offset,
) -> impl Iterator<Item = SyntaxNode> {
    node.token_at_offset(offset)
        .map(|token| token.parent_ancestors().collect_vec())
        .kmerge_by(|node1, node2| node1.span().len() < node2.span().len())
        .dedup_by(|node1, node2| node1 == node2)
}

pub fn ancestors_at_position(
    node: &SyntaxNode,
    position: tombi_text::Position,
) -> impl Iterator<Item = SyntaxNode> {
    node.token_at_position(position)
        .map(|token| token.parent_ancestors().collect_vec())
        .kmerge_by(|node1, node2| node1.span().len() <= node2.span().len())
        .dedup_by(|node1, node2| node1 == node2)
}
