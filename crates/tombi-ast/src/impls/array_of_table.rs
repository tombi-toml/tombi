use tombi_syntax::{SyntaxKind::*, T};
use tombi_toml_version::TomlVersion;

use crate::{
    ArrayOfTable, AstChildren, AstNode, DanglingCommentGroupOr, KeyValueGroup, TableOrArrayOfTable,
    TombiValueCommentDirective, support,
};

impl crate::ArrayOfTable {
    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        itertools::chain!(
            self.header_leading_comments()
                .filter_map(|comment| comment.get_tombi_value_directive()),
            self.header_trailing_comment()
                .into_iter()
                .filter_map(|comment| comment.get_tombi_value_directive()),
            self.dangling_comment_groups().flat_map(|comment_group| {
                comment_group
                    .into_comments()
                    .filter_map(|comment| comment.get_tombi_value_directive())
            })
        )
    }

    /// The leading comments of the array of table header.
    ///
    /// ```toml
    /// # This comment
    /// [[table]]
    /// ```
    pub fn header_leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        support::comment::leading_comments(self.syntax().children_with_tokens())
    }

    /// The trailing comment of the array of table header.
    ///
    /// ```toml
    /// [[table]]  # This comment
    /// ```
    pub fn header_trailing_comment(&self) -> Option<crate::TrailingComment> {
        support::comment::trailing_comment(self.syntax().children_with_tokens(), T!("]]"))
    }

    /// The dangling comments of the array of table (without key-value pairs).
    ///
    /// ```toml
    /// [[table]]
    /// # This comments
    /// # This comments
    ///
    /// # This comments
    /// # This comments
    ///
    /// key = "value"
    /// ```
    pub fn dangling_comment_groups(&self) -> impl Iterator<Item = crate::DanglingCommentGroup> {
        support::comment::dangling_comment_groups(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!("]]")))
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), LINE_BREAK)),
        )
    }

    pub fn key_value_groups(&self) -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>> {
        support::comment::dangling_comment_group_or(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!("]]")))
                .skip_while(|node_or_token| {
                    !matches!(node_or_token.kind(), LINE_BREAK | DANGLING_COMMENT_GROUP)
                }),
        )
    }

    pub fn contains_header(&self, position: tombi_text::Position) -> bool {
        self.double_bracket_start().unwrap().range().end <= position
            && position <= self.double_bracket_end().unwrap().range().start
    }

    /// Returns an iterator over the sub-tables of this table.
    ///
    /// ```toml
    /// [[foo]]  # <- This is a self array of table
    /// key = "value"
    ///
    /// [foo.bar]  # <- This is a subtable
    /// key = "value"
    ///
    /// [[foo.baz]]  # <- This is also a subtable
    /// key = true
    /// ```
    pub fn sub_tables(&self) -> impl Iterator<Item = TableOrArrayOfTable> + '_ {
        support::node::next_siblings_nodes(self)
            .skip(1)
            .take_while(|t: &TableOrArrayOfTable| {
                let Some(keys) = t.header().map(|header| header.keys()) else {
                    return false;
                };
                let Some(self_keys) = self.header().map(|header| header.keys()) else {
                    return false;
                };

                keys.starts_with(&self_keys) && keys.count() != self_keys.count()
            })
    }

    pub fn parent_table_or_array_of_table_keys(
        &self,
        toml_version: TomlVersion,
    ) -> impl Iterator<Item = AstChildren<crate::Key>> + '_ {
        support::node::prev_siblings_nodes(self)
            .filter_map(|node: TableOrArrayOfTable| node.header().map(|header| header.keys()))
            .take_while(move |keys| {
                match (
                    self.header().and_then(|header| header.keys().next()),
                    keys.clone().next(),
                ) {
                    (Some(a), Some(b)) => match (
                        a.try_to_raw_text(toml_version),
                        b.try_to_raw_text(toml_version),
                    ) {
                        (Ok(a), Ok(b)) => a == b,
                        _ => false,
                    },
                    _ => false,
                }
            })
            .filter(|keys| {
                self.header()
                    .map(|header_keys| header_keys.keys().starts_with(keys))
                    .unwrap_or_default()
            })
    }

    pub fn parrent_array_of_tables_keys(
        &self,
    ) -> impl Iterator<Item = AstChildren<crate::Key>> + '_ {
        support::node::prev_siblings_nodes(self)
            .filter_map(|node: ArrayOfTable| node.header().map(|header| header.keys()))
            .take_while(move |keys| {
                match (
                    self.header().and_then(|header| header.keys().next()),
                    keys.clone().next(),
                ) {
                    (Some(a), Some(b)) => match (
                        a.try_to_raw_text(TomlVersion::latest()),
                        b.try_to_raw_text(TomlVersion::latest()),
                    ) {
                        (Ok(a), Ok(b)) => a == b,
                        _ => false,
                    },
                    _ => false,
                }
            })
            .filter(|keys| {
                self.header()
                    .map(|header_keys| header_keys.keys().starts_with(keys))
                    .unwrap_or_default()
            })
    }
}
