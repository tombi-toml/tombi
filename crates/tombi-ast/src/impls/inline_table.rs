use itertools::Itertools;
use tombi_syntax::{SyntaxKind::*, T};
use tombi_toml_version::TomlVersion;

use crate::{support, AstNode, TombiValueCommentDirective};

impl crate::InlineTable {
    pub fn inner_begin_dangling_comments(&self) -> Vec<Vec<crate::BeginDanglingComment>> {
        support::node::begin_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| node.kind() != T!('{'))
                .skip(1), // skip '{'
        )
    }

    pub fn inner_end_dangling_comments(&self) -> Vec<Vec<crate::EndDanglingComment>> {
        support::node::end_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .take_while(|node| node.kind() != T!('}')),
        )
    }

    #[inline]
    pub fn inner_dangling_comments(&self) -> Vec<Vec<crate::DanglingComment>> {
        support::node::dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| node.kind() != T!('{'))
                .skip(1) // skip '{'
                .take_while(|node| node.kind() != T!('}')),
        )
    }

    #[inline]
    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        let mut comment_directives = vec![];

        if self.key_values().next().is_none() {
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

    pub fn key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::KeyValue, Option<crate::Comma>)> {
        self.key_values()
            .zip_longest(support::node::children::<crate::Comma>(self.syntax()))
            .map(|value_with_comma| match value_with_comma {
                itertools::EitherOrBoth::Both(value, comma) => (value, Some(comma)),
                itertools::EitherOrBoth::Left(value) => (value, None),
                itertools::EitherOrBoth::Right(_) => unreachable!(),
            })
    }

    pub fn should_be_multiline(&self, toml_version: TomlVersion) -> bool {
        match toml_version {
            TomlVersion::V1_0_0 => false,
            TomlVersion::V1_1_0_Preview => {
                self.has_trailing_comma_after_last_value()
                    || self.has_multiline_values(toml_version)
                    // || self.has_only_comments(toml_version)
                    || self.has_inner_comments()
            }
        }
    }

    pub fn has_trailing_comma_after_last_value(&self) -> bool {
        self.syntax()
            .children_with_tokens()
            .collect_vec()
            .into_iter()
            .rev()
            .skip_while(|item| item.kind() != T!('}'))
            .skip(1)
            .find(|item| !matches!(item.kind(), WHITESPACE | COMMENT | LINE_BREAK))
            .is_some_and(|it| it.kind() == T!(,))
    }

    pub fn has_multiline_values(&self, toml_version: TomlVersion) -> bool {
        self.key_values().any(|key_value| {
            key_value.value().is_some_and(|value| match value {
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
        })
    }

    #[inline]
    pub fn has_only_comments(&self, toml_version: TomlVersion) -> bool {
        match toml_version {
            TomlVersion::V1_0_0 => false,
            TomlVersion::V1_1_0_Preview => support::node::has_only_comments(
                self.syntax().children_with_tokens(),
                T!('{'),
                T!('}'),
            ),
        }
    }

    #[inline]
    pub fn has_inner_comments(&self) -> bool {
        support::node::has_inner_comments(self.syntax().children_with_tokens(), T!('{'), T!('}'))
    }
}
