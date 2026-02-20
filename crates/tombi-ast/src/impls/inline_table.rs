use itertools::Itertools;
use tombi_syntax::{SyntaxKind::*, T};
use tombi_toml_version::TomlVersion;

use crate::{
    AstNode, DanglingCommentGroupOr, KeyValueWithCommaGroup, TombiValueCommentDirective, support,
};

impl crate::InlineTable {
    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        itertools::chain!(
            self.brace_start_trailing_comment()
                .into_iter()
                .filter_map(|comment| comment.get_tombi_value_directive()),
            self.dangling_comment_groups()
                .flat_map(|comment_group| comment_group
                    .into_comments()
                    .filter_map(|comment| comment.get_tombi_value_directive()))
        )
    }

    /// The trailing comment of the inline table start brace.
    ///
    /// ```toml
    /// key = {  # This comment
    /// }
    /// ```
    #[inline]
    pub fn brace_start_trailing_comment(&self) -> Option<crate::TrailingComment> {
        support::comment::trailing_comment(self.syntax().children_with_tokens(), T!('{'))
    }

    /// The dangling comments of the inline table (without key-values).
    ///
    /// ```toml
    /// key = {
    ///     # This comments
    ///     # This comments
    ///
    ///     # This comments
    ///     # This comments
    ///
    ///     "value"
    /// }
    #[inline]
    pub fn dangling_comment_groups(&self) -> impl Iterator<Item = crate::DanglingCommentGroup> {
        support::comment::dangling_comment_groups(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!('{')))
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), COMMENT))
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), LINE_BREAK)),
        )
    }

    #[inline]
    pub fn key_value_with_comma_groups(
        &self,
    ) -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueWithCommaGroup>> {
        support::comment::dangling_comment_group_or(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!('{')))
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), COMMENT))
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), LINE_BREAK)),
        )
    }

    #[inline]
    pub fn should_be_multiline(&self, toml_version: TomlVersion) -> bool {
        if toml_version == TomlVersion::V1_0_0 {
            return false;
        }

        self.has_last_key_value_trailing_comma()
            || self.has_multiline_values(toml_version)
            || self.has_inner_comments()
    }

    #[inline]
    pub fn has_last_key_value_trailing_comma(&self) -> bool {
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

    #[inline]
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
    pub fn has_inner_comments(&self) -> bool {
        support::comment::has_inner_comments(self.syntax().children_with_tokens(), T!('{'), T!('}'))
    }
}
