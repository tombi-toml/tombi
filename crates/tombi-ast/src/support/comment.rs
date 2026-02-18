use itertools::Itertools;
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
pub fn dangling_comments<I: Iterator<Item = tombi_syntax::SyntaxElement>>(
    iter: I,
) -> Vec<Vec<crate::DanglingComment>> {
    group_comments(iter.take_while(|node| matches!(node.kind(), COMMENT | WHITESPACE | LINE_BREAK)))
}

#[inline]
pub fn begin_dangling_comments<I: Iterator<Item = tombi_syntax::SyntaxElement>>(
    iter: I,
) -> Vec<Vec<crate::BeginDanglingComment>> {
    group_comments(iter.take_while(|node| matches!(node.kind(), COMMENT | WHITESPACE | LINE_BREAK)))
}

#[inline]
pub fn end_dangling_comments<I: Iterator<Item = tombi_syntax::SyntaxElement>>(
    iter: I,
) -> Vec<Vec<crate::EndDanglingComment>> {
    let comment_iter = iter
        .collect_vec()
        .into_iter()
        .rev()
        .take_while(|node| matches!(node.kind(), COMMENT | WHITESPACE | LINE_BREAK))
        .collect_vec()
        .into_iter()
        .rev();

    // NOTE: If there is a whitespace comment at the beginning, it is treated as a empty comment group.
    //       This allows us to insert a line break at the beginning when formatting.
    let comment_groups = group_comments(comment_iter.clone());
    if comment_groups.is_empty()
        || comment_iter
            .skip(1) // skip LineBreak
            .find(|node| node.kind() != WHITESPACE)
            .is_some_and(|node| node.kind() == COMMENT)
    {
        // No new line break at the beginning
        //
        // ```toml
        // [foo]
        // bar = 1
        // # end dangling comment1
        // # end dangling comment2
        // ```
        comment_groups
    } else {
        // New line break at the beginning
        //
        // ```toml
        // [foo]
        // bar = 1
        //
        // # end dangling comment1
        // # end dangling comment2
        // ```
        let mut result = Vec::with_capacity(comment_groups.len() + 1);
        result.push(Vec::with_capacity(0));
        result.extend(comment_groups);
        result
    }
}

/// Group comments with empty line breaks.
#[inline]
fn group_comments<T, I: Iterator<Item = tombi_syntax::SyntaxElement>>(iter: I) -> Vec<Vec<T>>
where
    T: From<crate::Comment>,
{
    let mut is_new_group = false;
    iter.fold(Vec::new(), |mut acc, node_or_token| {
        match node_or_token {
            SyntaxElement::Token(token) => match token.kind() {
                COMMENT => {
                    let comment = crate::Comment::cast(token).unwrap();
                    if let Some(last_group) = acc.last_mut() {
                        if is_new_group {
                            acc.push(vec![comment.into()]);
                        } else {
                            last_group.push(comment.into());
                        }
                    } else {
                        acc.push(vec![comment.into()]);
                    }
                    is_new_group = false;
                }
                LINE_BREAK => {
                    if token.next_sibling_or_token().is_some_and(|next| {
                        if next.kind() == WHITESPACE {
                            next.next_sibling_or_token()
                                .is_some_and(|next| next.kind() == LINE_BREAK)
                        } else {
                            next.kind() == LINE_BREAK
                        }
                    }) && acc.last().is_some_and(|last_group| !last_group.is_empty())
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
