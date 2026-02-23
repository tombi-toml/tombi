use itertools::Itertools;
use tombi_syntax::T;
use tombi_toml_version::TomlVersion;

use crate::{
    AstNode, DanglingCommentGroupOr, KeyValueWithCommaGroup, TombiValueCommentDirective,
    support::{self, comment::skip_trailing_comment},
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
                    .filter_map(|comment| comment.get_tombi_value_directive())),
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
        support::comment::dangling_comment_groups(skip_trailing_comment(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!('{')))
                .skip(1)
                .peekable(),
        ))
    }

    #[inline]
    pub fn key_value_with_comma_groups(
        &self,
    ) -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueWithCommaGroup>> {
        support::comment::dangling_comment_group_or(skip_trailing_comment(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!('{')))
                .skip(1)
                .peekable(),
        ))
    }

    #[inline]
    pub fn key_values(&self) -> impl Iterator<Item = crate::KeyValue> {
        self.key_value_with_comma_groups()
            .filter_map(DanglingCommentGroupOr::into_item_group)
            .flat_map(KeyValueWithCommaGroup::into_key_values)
    }

    #[inline]
    pub fn key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::KeyValue, Option<crate::Comma>)> {
        self.key_value_with_comma_groups()
            .filter_map(|group| {
                group
                    .into_item_group()
                    .map(|key_value_group| key_value_group.key_values_with_comma().collect_vec())
            })
            .flatten()
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
        self.key_value_with_comma_groups()
            .collect_vec()
            .into_iter()
            .rev()
            .find_map(|group| match group {
                DanglingCommentGroupOr::DanglingCommentGroup(_) => None,
                DanglingCommentGroupOr::ItemGroup(key_value_group) => key_value_group
                    .key_values_with_comma()
                    .last()
                    .map(|(_, comma)| comma.is_some()),
            })
            .unwrap_or(false)
    }

    #[inline]
    pub fn has_multiline_values(&self, toml_version: TomlVersion) -> bool {
        for group in self.key_value_with_comma_groups() {
            let key_value_group = match group {
                DanglingCommentGroupOr::DanglingCommentGroup(_) => return true,
                DanglingCommentGroupOr::ItemGroup(key_value_group) => key_value_group,
            };

            for key_value in key_value_group.key_values() {
                let is_multiline = key_value.value().is_some_and(|value| match value {
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
                });

                if is_multiline {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    pub fn has_inner_comments(&self) -> bool {
        support::comment::has_inner_comments(self.syntax().children_with_tokens(), T!('{'), T!('}'))
    }
}
