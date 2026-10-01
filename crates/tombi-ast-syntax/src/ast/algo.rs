use itertools::Itertools;
use tombi_ast_syntax::SyntaxNode;

pub fn ancestors_at_offset<'t>(
    node: &SyntaxNode<'t>,
    offset: tombi_text::Offset,
) -> impl Iterator<Item = SyntaxNode<'t>> + use<'t> {
    let nodes = match node.token_at_offset(offset) {
        crate::TokenAtOffset::None => Vec::new(),
        crate::TokenAtOffset::Single(token) => token.parent_ancestors().collect(),
        crate::TokenAtOffset::Between(left, right) => [
            left.parent_ancestors().collect_vec(),
            right.parent_ancestors().collect_vec(),
        ]
        .into_iter()
        .kmerge_by(|node1, node2| node1.span().len() <= node2.span().len())
        .dedup_by(|node1, node2| node1 == node2)
        .collect(),
    };
    nodes.into_iter()
}
