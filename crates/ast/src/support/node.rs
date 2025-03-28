use itertools::Itertools;
use syntax::{SyntaxElement, SyntaxKind, SyntaxKind::*};

use crate::{AstChildren, AstNode, AstToken};

#[inline]
pub fn child<N: AstNode>(parent: &syntax::SyntaxNode) -> Option<N> {
    parent.children().find_map(N::cast)
}

#[inline]
pub fn children<N: AstNode>(parent: &syntax::SyntaxNode) -> AstChildren<N> {
    AstChildren::new(parent)
}

#[inline]
pub fn token(parent: &syntax::SyntaxNode, kind: syntax::SyntaxKind) -> Option<syntax::SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == kind)
}

#[inline]
pub fn leading_comments<I: Iterator<Item = syntax::SyntaxElement>>(
    iter: I,
) -> impl Iterator<Item = crate::LeadingComment> {
    iter.take_while(|node| matches!(node.kind(), COMMENT | LINE_BREAK | WHITESPACE))
        .filter_map(|node_or_token| match node_or_token {
            SyntaxElement::Token(token) => crate::Comment::cast(token).map(Into::into),
            SyntaxElement::Node(_) => None,
        })
}

#[inline]
pub fn tailing_comment<I: Iterator<Item = syntax::SyntaxElement>>(
    iter: I,
    end: syntax::SyntaxKind,
) -> Option<crate::TailingComment> {
    let mut iter = iter
        .skip_while(|item| item.kind() != end && item.kind() != EOF)
        .skip(1);

    match iter.next()? {
        SyntaxElement::Token(token) if token.kind() == COMMENT => {
            crate::Comment::cast(token).map(Into::into)
        }
        SyntaxElement::Token(token) if token.kind() == WHITESPACE => {
            iter.next().and_then(|node_or_token| match node_or_token {
                SyntaxElement::Token(token) if token.kind() == COMMENT => {
                    crate::Comment::cast(token).map(Into::into)
                }
                _ => None,
            })
        }
        _ => None,
    }
}

#[inline]
pub fn dangling_comments<I: Iterator<Item = syntax::SyntaxElement>>(
    iter: I,
) -> Vec<Vec<crate::DanglingComment>> {
    group_comments(iter.take_while(|node| matches!(node.kind(), COMMENT | WHITESPACE | LINE_BREAK)))
}

#[inline]
pub fn begin_dangling_comments<I: Iterator<Item = syntax::SyntaxElement>>(
    iter: I,
) -> Vec<Vec<crate::BeginDanglingComment>> {
    group_comments(iter.take_while(|node| matches!(node.kind(), COMMENT | WHITESPACE | LINE_BREAK)))
}

#[inline]
pub fn end_dangling_comments<I: Iterator<Item = syntax::SyntaxElement>>(
    iter: I,
) -> Vec<Vec<crate::EndDanglingComment>> {
    group_comments(
        iter.collect_vec()
            .into_iter()
            .rev()
            .take_while(|node| matches!(node.kind(), COMMENT | WHITESPACE | LINE_BREAK))
            .collect_vec()
            .into_iter()
            .rev(),
    )
}

/// Group comments with empty line breaks.
#[inline]
fn group_comments<T, I: Iterator<Item = syntax::SyntaxElement>>(iter: I) -> Vec<Vec<T>>
where
    T: From<crate::Comment>,
{
    let mut is_new_group = false;
    iter.fold(Vec::new(), |mut acc, node_or_token| {
        match node_or_token {
            SyntaxElement::Token(token) => match token.kind() {
                COMMENT => {
                    if let Some(last_group) = acc.last_mut() {
                        if let Some(comment) = crate::Comment::cast(token) {
                            if is_new_group {
                                acc.push(vec![comment.into()]);
                            } else {
                                last_group.push(comment.into());
                            }
                        }
                    } else if let Some(comment) = crate::Comment::cast(token) {
                        acc.push(vec![comment.into()]);
                    }
                    is_new_group = false;
                }
                LINE_BREAK => {
                    if token
                        .next_sibling_or_token()
                        .is_some_and(|next| next.kind() == LINE_BREAK)
                        && acc.last().is_some_and(|last_group| !last_group.is_empty())
                    {
                        is_new_group = true;
                    }
                }
                WHITESPACE => {}
                _ => unreachable!("unexpected token {:?}", token.kind()),
            },
            SyntaxElement::Node(_) => {}
        }
        acc
    })
}

pub fn has_only_comments<I: Iterator<Item = syntax::SyntaxElement>>(
    iter: I,
    start: SyntaxKind,
    end: SyntaxKind,
) -> bool {
    iter.skip_while(|node| node.kind() != start)
        .skip(1)
        .take_while(|node| node.kind() != end)
        .all(|node| matches!(node.kind(), WHITESPACE | COMMENT | LINE_BREAK))
}

#[inline]
pub fn has_inner_comments<I: Iterator<Item = syntax::SyntaxElement>>(
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
                    syntax::SyntaxElement::Node(node) => node
                        .children_with_tokens()
                        .any(|node| node.kind() == COMMENT),
                    _ => false,
                }
        })
}

pub fn prev_siblings_nodes<N: AstNode, T: AstNode>(node: &N) -> impl Iterator<Item = T> {
    node.syntax()
        .siblings(syntax::Direction::Prev)
        .skip(1)
        .filter_map(T::cast)
}

pub fn next_siblings_nodes<N: AstNode, T: AstNode>(node: &N) -> impl Iterator<Item = T> {
    node.syntax()
        .siblings(syntax::Direction::Next)
        .filter_map(T::cast)
}
