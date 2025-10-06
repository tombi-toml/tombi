use std::{borrow::Cow, fmt, marker::PhantomData, ops::Range};

use itertools::Itertools;

use super::{RedElement, RedToken};
use crate::{
    cursor,
    green::{GreenNode, GreenNodeData},
    red::{Preorder, RedElementChildren, RedNodeChildren, RedPreorderWithTokens},
    Direction, Language, NodeOrToken, SyntaxText, TokenAtOffset, WalkEvent,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RedNode<L: Language> {
    raw: cursor::SyntaxNode,
    _p: PhantomData<L>,
}

impl<L: Language> fmt::Debug for RedNode<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let mut level = 0;
            for event in self.preorder_with_tokens() {
                match event {
                    WalkEvent::Enter(element) => {
                        for _ in 0..level {
                            write!(f, "  ")?;
                        }
                        match element {
                            NodeOrToken::Node(node) => writeln!(f, "{node:?}")?,
                            NodeOrToken::Token(token) => writeln!(f, "{token:?}")?,
                        }
                        level += 1;
                    }
                    WalkEvent::Leave(_) => level -= 1,
                }
            }
            debug_assert_eq!(level, 0);
            Ok(())
        } else {
            write!(f, "{:?} @{} @{}", self.kind(), self.span(), self.range())
        }
    }
}

impl<L: Language> fmt::Display for RedNode<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}

impl<L: Language> RedNode<L> {
    pub fn new_root(green: GreenNode) -> RedNode<L> {
        RedNode::from(cursor::SyntaxNode::new_root(green))
    }
    pub fn new_root_mut(green: GreenNode) -> RedNode<L> {
        RedNode::from(cursor::SyntaxNode::new_root_mut(green))
    }

    /// Returns a green tree, equal to the green tree this node
    /// belongs to, except with this node substituted. The complexity
    /// of the operation is proportional to the depth of the tree.
    pub fn replace_with(&self, replacement: GreenNode) -> GreenNode {
        self.raw.replace_with(replacement)
    }

    pub fn kind(&self) -> L::Kind {
        L::kind_from_raw(self.raw.kind())
    }

    pub fn span(&self) -> tombi_text::Span {
        self.raw.span()
    }

    pub fn range(&self) -> tombi_text::Range {
        self.raw.range()
    }

    pub fn index(&self) -> usize {
        self.raw.index()
    }

    pub fn text(&self) -> SyntaxText {
        self.raw.text()
    }

    pub fn green(&self) -> Cow<'_, GreenNodeData> {
        self.raw.green()
    }

    pub fn parent(&self) -> Option<RedNode<L>> {
        self.raw.parent().map(Self::from)
    }

    pub fn ancestors(&self) -> impl Iterator<Item = RedNode<L>> {
        self.raw.ancestors().map(RedNode::from)
    }

    pub fn children(&self) -> RedNodeChildren<L> {
        self.raw.children().into()
    }

    pub fn children_with_tokens(&self) -> RedElementChildren<L> {
        self.raw.children_with_tokens().into()
    }

    pub fn first_child(&self) -> Option<RedNode<L>> {
        self.raw.first_child().map(Self::from)
    }
    pub fn last_child(&self) -> Option<RedNode<L>> {
        self.raw.last_child().map(Self::from)
    }

    pub fn first_child_or_token(&self) -> Option<RedElement<L>> {
        self.raw.first_child_or_token().map(NodeOrToken::from)
    }
    pub fn last_child_or_token(&self) -> Option<RedElement<L>> {
        self.raw.last_child_or_token().map(NodeOrToken::from)
    }

    pub fn next_sibling(&self) -> Option<RedNode<L>> {
        self.raw.next_sibling().map(Self::from)
    }
    pub fn prev_sibling(&self) -> Option<RedNode<L>> {
        self.raw.prev_sibling().map(Self::from)
    }

    pub fn next_sibling_or_token(&self) -> Option<RedElement<L>> {
        self.raw.next_sibling_or_token().map(NodeOrToken::from)
    }
    pub fn prev_sibling_or_token(&self) -> Option<RedElement<L>> {
        self.raw.prev_sibling_or_token().map(NodeOrToken::from)
    }

    /// Return the leftmost token in the subtree of this node.
    pub fn first_token(&self) -> Option<RedToken<L>> {
        self.raw.first_token().map(RedToken::from)
    }
    /// Return the rightmost token in the subtree of this node.
    pub fn last_token(&self) -> Option<RedToken<L>> {
        self.raw.last_token().map(RedToken::from)
    }

    pub fn siblings(&self, direction: Direction) -> impl Iterator<Item = RedNode<L>> {
        self.raw.siblings(direction).map(RedNode::from)
    }

    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = RedElement<L>> {
        self.raw
            .siblings_with_tokens(direction)
            .map(RedElement::from)
    }

    pub fn descendants(&self) -> impl Iterator<Item = RedNode<L>> {
        self.raw.descendants().map(RedNode::from)
    }

    pub fn descendants_with_tokens(&self) -> impl Iterator<Item = RedElement<L>> {
        self.raw.descendants_with_tokens().map(NodeOrToken::from)
    }

    /// Traverse the subtree rooted at the current node (including the current
    /// node) in preorder, excluding tokens.
    pub fn preorder(&self) -> Preorder<L> {
        self.raw.preorder().into()
    }

    /// Traverse the subtree rooted at the current node (including the current
    /// node) in preorder, including tokens.
    pub fn preorder_with_tokens(&self) -> RedPreorderWithTokens<L> {
        self.raw.preorder_with_tokens().into()
    }

    /// Find a token in the subtree corresponding to this node, which covers the offset.
    /// Precondition: offset must be within node's span.
    pub fn token_at_offset(&self, offset: tombi_text::Offset) -> TokenAtOffset<RedToken<L>> {
        self.raw.token_at_offset(offset).map(RedToken::from)
    }

    pub fn token_at_position(&self, position: tombi_text::Position) -> TokenAtOffset<RedToken<L>> {
        self.raw.token_at_position(position).map(RedToken::from)
    }

    /// Return the deepest node or token in the current subtree that fully
    /// contains the span. If the span is empty and is contained in two leaf
    /// nodes, either one can be returned. Precondition: span must be contained
    /// within the current node
    pub fn covering_element(&self, span: tombi_text::Span) -> RedElement<L> {
        NodeOrToken::from(self.raw.covering_element(span))
    }

    /// Finds a [`SyntaxElement`] which intersects with a given `span`. If
    /// there are several intersecting elements, any one can be returned.
    ///
    /// The method uses binary search internally, so it's complexity is
    /// `O(log(N))` where `N = self.children_with_tokens().count()`.
    pub fn child_or_token_at_span(&self, span: tombi_text::Span) -> Option<RedElement<L>> {
        self.raw.child_or_token_at_span(span).map(RedElement::from)
    }

    /// Returns an independent copy of the subtree rooted at this node.
    ///
    /// The parent of the returned node will be `None`, the start offset will be
    /// zero, but, otherwise, it'll be equivalent to the source node.
    pub fn clone_subtree(&self) -> RedNode<L> {
        RedNode::from(self.raw.clone_subtree())
    }

    pub fn clone_for_update(&self) -> RedNode<L> {
        RedNode::from(self.raw.clone_for_update())
    }

    pub fn is_mutable(&self) -> bool {
        self.raw.is_mutable()
    }

    pub fn detach(&self) {
        self.raw.detach()
    }

    pub fn splice_children(&self, to_delete: Range<usize>, to_insert: Vec<RedElement<L>>) {
        let to_insert = to_insert
            .into_iter()
            .map(cursor::SyntaxElement::from)
            .collect_vec();
        self.raw.splice_children(to_delete, to_insert)
    }
}

impl<L: Language> From<cursor::SyntaxNode> for RedNode<L> {
    fn from(raw: cursor::SyntaxNode) -> RedNode<L> {
        RedNode {
            raw,
            _p: PhantomData,
        }
    }
}

impl<L: Language> From<RedNode<L>> for cursor::SyntaxNode {
    fn from(node: RedNode<L>) -> cursor::SyntaxNode {
        node.raw
    }
}
