use itertools::Itertools;
use tombi_syntax::{SyntaxKind::*, T};
use tombi_toml_version::TomlVersion;

use crate::{support, AstNode, TombiValueCommentDirective};

impl crate::Array {
    #[inline]
    pub fn inner_begin_dangling_comments(&self) -> Vec<Vec<crate::BeginDanglingComment>> {
        support::node::begin_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| node.kind() != T!('['))
                .skip(1), // skip '['
        )
    }

    #[inline]
    pub fn inner_end_dangling_comments(&self) -> Vec<Vec<crate::EndDanglingComment>> {
        support::node::end_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .take_while(|node| node.kind() != T!(']')),
        )
    }

    #[inline]
    pub fn inner_dangling_comments(&self) -> Vec<Vec<crate::DanglingComment>> {
        support::node::dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| node.kind() != T!('['))
                .skip(1) // skip '['
                .take_while(|node| node.kind() != T!(']')),
        )
    }

    #[inline]
    pub fn values_with_comma(&self) -> impl Iterator<Item = (crate::Value, Option<crate::Comma>)> {
        self.values()
            .zip_longest(support::node::children::<crate::Comma>(self.syntax()))
            .filter_map(|value_with_comma| match value_with_comma {
                itertools::EitherOrBoth::Both(value, comma) => Some((value, Some(comma))),
                itertools::EitherOrBoth::Left(value) => Some((value, None)),
                itertools::EitherOrBoth::Right(_) => None,
            })
    }

    #[inline]
    pub fn value_or_key_values(&self) -> impl Iterator<Item = crate::ValueOrKeyValue> {
        support::node::children(self.syntax())
    }

    #[inline]
    pub fn value_or_key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::ValueOrKeyValue, Option<crate::Comma>)> {
        self.value_or_key_values()
            .zip_longest(support::node::children::<crate::Comma>(self.syntax()))
            .filter_map(|value_or_key_with_comma| match value_or_key_with_comma {
                itertools::EitherOrBoth::Both(value_or_key, comma) => {
                    Some((value_or_key, Some(comma)))
                }
                itertools::EitherOrBoth::Left(value_or_key) => Some((value_or_key, None)),
                itertools::EitherOrBoth::Right(_) => None,
            })
    }

    #[inline]
    pub fn should_be_multiline(&self, toml_version: TomlVersion) -> bool {
        self.has_last_value_trailing_comma()
            || self.has_multiline_values(toml_version)
            // || self.has_only_comments()
            || self.has_inner_comments()
    }

    #[inline]
    pub fn has_last_value_trailing_comma(&self) -> bool {
        self.syntax()
            .children_with_tokens()
            .collect_vec()
            .into_iter()
            .rev()
            .skip_while(|item| item.kind() != T!(']'))
            .skip(1)
            .find(|item| !matches!(item.kind(), WHITESPACE | COMMENT | LINE_BREAK))
            .is_some_and(|it| it.kind() == T!(,))
    }

    #[inline]
    pub fn has_multiline_values(&self, toml_version: TomlVersion) -> bool {
        self.values().any(|value| match value {
            crate::Value::Array(array) => array.should_be_multiline(toml_version),
            crate::Value::InlineTable(inline_table) => {
                inline_table.should_be_multiline(toml_version)
            }
            crate::Value::MultiLineBasicString(string) => {
                string.token().unwrap().text().contains('\n')
            }
            crate::Value::MultiLineLiteralString(string) => {
                string.token().unwrap().text().contains('\n')
            }
            _ => false,
        })
    }

    #[inline]
    pub fn has_only_comments(&self) -> bool {
        support::node::has_only_comments(self.syntax().children_with_tokens(), T!('['), T!(']'))
    }

    #[inline]
    pub fn has_inner_comments(&self) -> bool {
        support::node::has_inner_comments(self.syntax().children_with_tokens(), T!('['), T!(']'))
    }

    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        let mut comment_directives = vec![];

        if self.values().next().is_none() {
            for comments in self.inner_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        comment_directives.push(comment_directive);
                    }
                }
            }
        } else {
            for comments in self.inner_begin_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        comment_directives.push(comment_directive);
                    }
                }
            }
            for comments in self.inner_end_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        comment_directives.push(comment_directive);
                    }
                }
            }
        }

        comment_directives.into_iter()
    }
}
