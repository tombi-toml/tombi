use tombi_syntax::{
    SyntaxElement,
    SyntaxKind::{self, *},
};

use crate::{AstNode, AstToken, DanglingCommentGroupOr};

#[inline]
pub fn dangling_comment_groups<I: Iterator<Item = tombi_syntax::SyntaxElement>>(
    iter: I,
) -> impl Iterator<Item = crate::DanglingCommentGroup> {
    iter.take_while(|node_or_token| {
        matches!(node_or_token.kind(), DANGLING_COMMENT_GROUP | LINE_BREAK)
    })
    .filter_map(|node_or_token| match node_or_token {
        SyntaxElement::Node(node) => crate::DanglingCommentGroup::cast(node),
        SyntaxElement::Token(_) => None,
    })
}

#[inline]
pub fn dangling_comment_group_or<T: AstNode, I: Iterator<Item = tombi_syntax::SyntaxElement>>(
    iter: I,
) -> impl Iterator<Item = DanglingCommentGroupOr<T>> {
    iter.filter_map(|node_or_token| match node_or_token {
        SyntaxElement::Node(node) => {
            if crate::DanglingCommentGroup::can_cast(node.kind()) {
                crate::DanglingCommentGroup::cast(node)
                    .map(DanglingCommentGroupOr::DanglingCommentGroup)
            } else if let Some(item) = T::cast(node) {
                Some(DanglingCommentGroupOr::ItemGroup(item))
            } else {
                None
            }
        }
        SyntaxElement::Token(_) => None,
    })
}

#[inline]
pub fn leading_comments<I: Iterator<Item = tombi_syntax::SyntaxElement>>(
    iter: I,
) -> impl Iterator<Item = crate::LeadingComment> {
    iter.take_while(|node_or_token| {
        matches!(node_or_token.kind(), COMMENT | LINE_BREAK | WHITESPACE)
    })
    .filter_map(|node_or_token| match node_or_token {
        SyntaxElement::Token(token) => crate::LeadingComment::cast(token),
        SyntaxElement::Node(_) => None,
    })
}

#[inline]
pub fn trailing_comment<I: Iterator<Item = tombi_syntax::SyntaxElement>>(
    iter: I,
    end: tombi_syntax::SyntaxKind,
) -> Option<crate::TrailingComment> {
    let mut iter = iter
        .skip_while(|item| item.kind() != end && item.kind() != EOF)
        .skip(1);

    match iter.next()? {
        SyntaxElement::Token(token) if token.kind() == COMMENT => {
            crate::TrailingComment::cast(token)
        }
        SyntaxElement::Token(token) if token.kind() == WHITESPACE => {
            iter.next().and_then(|node_or_token| match node_or_token {
                SyntaxElement::Token(token) if token.kind() == COMMENT => {
                    crate::TrailingComment::cast(token)
                }
                _ => None,
            })
        }
        _ => None,
    }
}

#[inline]
pub fn has_inner_comments<I: Iterator<Item = tombi_syntax::SyntaxElement>>(
    iter: I,
    start: SyntaxKind,
    end: SyntaxKind,
) -> bool {
    iter.skip_while(|node| node.kind() != start)
        .skip(1)
        .take_while(|node| node.kind() != end)
        .any(|node| {
            node.kind() == COMMENT
                || match node {
                    tombi_syntax::SyntaxElement::Node(node) => node
                        .children_with_tokens()
                        .any(|node| node.kind() == COMMENT),
                    _ => false,
                }
        })
}
