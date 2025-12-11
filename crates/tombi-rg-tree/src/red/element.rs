use std::iter;

use super::{node::RedNode, token::RedToken};
use crate::{Language, NodeOrToken, cursor};

pub type RedElement<L> = NodeOrToken<RedNode<L>, RedToken<L>>;

impl<L: Language> From<RedNode<L>> for RedElement<L> {
    fn from(node: RedNode<L>) -> RedElement<L> {
        NodeOrToken::Node(node)
    }
}

impl<L: Language> From<RedToken<L>> for RedElement<L> {
    fn from(token: RedToken<L>) -> RedElement<L> {
        NodeOrToken::Token(token)
    }
}

impl<L: Language> RedElement<L> {
    pub fn span(&self) -> tombi_text::Span {
        match self {
            NodeOrToken::Node(it) => it.span(),
            NodeOrToken::Token(it) => it.span(),
        }
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            NodeOrToken::Node(it) => it.range(),
            NodeOrToken::Token(it) => it.range(),
        }
    }

    pub fn index(&self) -> usize {
        match self {
            NodeOrToken::Node(it) => it.index(),
            NodeOrToken::Token(it) => it.index(),
        }
    }

    pub fn kind(&self) -> L::Kind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    pub fn parent(&self) -> Option<RedNode<L>> {
        match self {
            NodeOrToken::Node(it) => it.parent(),
            NodeOrToken::Token(it) => it.parent(),
        }
    }

    pub fn ancestors(&self) -> impl Iterator<Item = RedNode<L>> {
        let first = match self {
            NodeOrToken::Node(it) => Some(it.clone()),
            NodeOrToken::Token(it) => it.parent(),
        };
        iter::successors(first, RedNode::parent)
    }

    pub fn next_sibling_or_token(&self) -> Option<RedElement<L>> {
        match self {
            NodeOrToken::Node(it) => it.next_sibling_or_token(),
            NodeOrToken::Token(it) => it.next_sibling_or_token(),
        }
    }
    pub fn prev_sibling_or_token(&self) -> Option<RedElement<L>> {
        match self {
            NodeOrToken::Node(it) => it.prev_sibling_or_token(),
            NodeOrToken::Token(it) => it.prev_sibling_or_token(),
        }
    }
    pub fn detach(&self) {
        match self {
            NodeOrToken::Node(it) => it.detach(),
            NodeOrToken::Token(it) => it.detach(),
        }
    }
}

impl<L: Language> From<cursor::SyntaxElement> for RedElement<L> {
    fn from(raw: cursor::SyntaxElement) -> RedElement<L> {
        match raw {
            NodeOrToken::Node(it) => NodeOrToken::Node(it.into()),
            NodeOrToken::Token(it) => NodeOrToken::Token(it.into()),
        }
    }
}

impl<L: Language> From<RedElement<L>> for cursor::SyntaxElement {
    fn from(element: RedElement<L>) -> cursor::SyntaxElement {
        match element {
            NodeOrToken::Node(it) => NodeOrToken::Node(it.into()),
            NodeOrToken::Token(it) => NodeOrToken::Token(it.into()),
        }
    }
}
