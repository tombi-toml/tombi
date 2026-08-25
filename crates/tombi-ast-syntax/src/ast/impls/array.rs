use itertools::Itertools;
use tombi_ast_syntax::T;
use tombi_toml_version::TomlVersion;

use crate::{
    AstNode, DanglingCommentGroupOr, TombiValueCommentDirective, ValueWithCommaGroup,
    support::{self, comment::skip_trailing_comment},
};

impl crate::Array {
    pub fn parent_key_value(&self) -> Option<crate::KeyValue> {
        self.syntax().parent().and_then(crate::KeyValue::cast)
    }

    /// Returns the comma immediately following the array item containing
    /// `position`. Invalid nodes are inspected so this also works while the
    /// user is typing incomplete TOML.
    pub fn comma_after(
        &self,
        position: tombi_text::Position,
    ) -> Option<tombi_ast_syntax::SyntaxToken> {
        let following = self
            .syntax()
            .child_elements()
            .skip_while(|element| !element.range().contains(position))
            .nth(1)?;
        match following {
            tombi_ast_syntax::SyntaxElement::Node(node)
                if node.kind() == tombi_ast_syntax::SyntaxKind::COMMA =>
            {
                node.first_token()
            }
            tombi_ast_syntax::SyntaxElement::Node(node)
                if node.kind() == tombi_ast_syntax::SyntaxKind::INVALID_TOKEN =>
            {
                node.first_token()
                    .filter(|token| token.kind() == tombi_ast_syntax::SyntaxKind::COMMA)
            }
            tombi_ast_syntax::SyntaxElement::Token(token)
                if token.kind() == tombi_ast_syntax::SyntaxKind::COMMA =>
            {
                Some(token)
            }
            _ => None,
        }
    }

    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        itertools::chain!(
            self.bracket_start_trailing_comment()
                .into_iter()
                .filter_map(|comment| comment.get_tombi_value_directive()),
            self.dangling_comment_groups()
                .flat_map(|comment_group| comment_group
                    .into_comments()
                    .filter_map(|comment| comment.get_tombi_value_directive())),
        )
    }

    /// The trailing comment of the array start bracket.
    ///
    /// ```toml
    /// key = [  # This comment
    /// ]
    /// ```
    #[inline]
    pub fn bracket_start_trailing_comment(&self) -> Option<crate::TrailingComment> {
        support::comment::trailing_comment(self.syntax().child_elements(), T!('['))
    }

    /// The dangling comments of the array (without values).
    ///
    /// ```toml
    /// key = [
    ///     # This comments
    ///     # This comments
    ///
    ///     # This comments
    ///     # This comments
    ///
    ///     "value"
    /// ]
    #[inline]
    pub fn dangling_comment_groups(&self) -> impl Iterator<Item = crate::DanglingCommentGroup> {
        support::comment::dangling_comment_groups(skip_trailing_comment(
            self.syntax()
                .child_elements()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!('[')))
                .skip(1)
                .peekable(),
        ))
    }

    #[inline]
    pub fn value_with_comma_groups(
        &self,
    ) -> impl Iterator<Item = DanglingCommentGroupOr<ValueWithCommaGroup>> {
        support::comment::dangling_comment_group_or(skip_trailing_comment(
            self.syntax()
                .child_elements()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!('[')))
                .skip(1)
                .peekable(),
        ))
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = crate::Value> {
        self.value_with_comma_groups()
            .filter_map(DanglingCommentGroupOr::into_item_group)
            .flat_map(ValueWithCommaGroup::into_values)
    }

    #[inline]
    pub fn values_with_comma(&self) -> impl Iterator<Item = (crate::Value, Option<crate::Comma>)> {
        self.value_with_comma_groups()
            .filter_map(DanglingCommentGroupOr::into_item_group)
            .flat_map(ValueWithCommaGroup::into_values_with_comma)
    }

    #[inline]
    pub fn value_or_key_values(&self) -> impl Iterator<Item = crate::ValueOrKeyValue> {
        self.syntax()
            .child_nodes()
            .filter_map(crate::ValueOrKeyValue::cast)
    }

    #[inline]
    pub fn value_or_key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::ValueOrKeyValue, Option<crate::Comma>)> {
        self.value_or_key_values()
            .zip_longest(self.syntax().child_nodes().filter_map(crate::Comma::cast))
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
            || self.has_inner_comments()
    }

    #[inline]
    pub fn has_last_value_trailing_comma(&self) -> bool {
        self.value_with_comma_groups()
            .collect_vec()
            .into_iter()
            .rev()
            .find_map(|group| match group {
                DanglingCommentGroupOr::DanglingCommentGroup(_) => None,
                DanglingCommentGroupOr::ItemGroup(value_group) => value_group
                    .values_with_comma()
                    .last()
                    .map(|(_, comma)| comma.is_some()),
            })
            .unwrap_or_default()
    }

    #[inline]
    pub fn has_multiline_values(&self, toml_version: TomlVersion) -> bool {
        for group in self.value_with_comma_groups() {
            let value_group = match group {
                DanglingCommentGroupOr::DanglingCommentGroup(_) => return true,
                DanglingCommentGroupOr::ItemGroup(value_group) => value_group,
            };

            for value in value_group.values() {
                let is_multiline = match value {
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
                };

                if is_multiline {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    pub fn has_inner_comments(&self) -> bool {
        support::comment::has_inner_comments(self.syntax().child_elements(), T!('['), T!(']'))
    }
}
