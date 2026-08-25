use tombi_ast_syntax::{SyntaxKind::*, T};
use tombi_toml_version::TomlVersion;

use crate::{
    ArrayOfTable, AstNode, DanglingCommentGroupOr, KeyValueGroup, TableOrArrayOfTable,
    TombiValueCommentDirective, support,
};

impl crate::Table {
    /// Range from the opening bracket through the last non-trivia element
    /// owned directly by this table.
    pub fn content_range(&self) -> Option<tombi_text::Range> {
        let mut elements = self.syntax().child_elements();
        let first = elements.find(|element| element.kind() == T!('['))?;
        let last = self
            .syntax()
            .child_elements()
            .filter(|element| {
                !matches!(
                    element.kind(),
                    tombi_ast_syntax::SyntaxKind::WHITESPACE
                        | tombi_ast_syntax::SyntaxKind::LINE_BREAK
                )
            })
            .last()?;
        Some(tombi_text::Range::new(
            first.range().start,
            last.range().end,
        ))
    }

    #[inline]
    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        itertools::chain!(
            self.header_leading_comments()
                .filter_map(|comment| comment.get_tombi_value_directive()),
            self.header_trailing_comment()
                .into_iter()
                .filter_map(|comment| comment.get_tombi_value_directive()),
            self.dangling_comment_groups()
                .flat_map(|comment_group| comment_group
                    .into_comments()
                    .filter_map(|comment| comment.get_tombi_value_directive()))
        )
    }

    /// The leading comments of the table header.
    ///
    /// ```toml
    /// # This comment
    /// [table]
    /// ```
    #[inline]
    pub fn header_leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        support::comment::leading_comments(self.syntax().child_elements())
    }

    /// The trailing comment of the table header.
    ///
    /// ```toml
    /// [table]  # This comment
    /// ```
    #[inline]
    pub fn header_trailing_comment(&self) -> Option<crate::TrailingComment> {
        support::comment::trailing_comment(self.syntax().child_elements(), T!(']'))
    }

    /// The dangling comments of the table (without key-value pairs).
    ///
    /// ```toml
    /// [table]
    /// # This comments
    /// # This comments
    ///
    /// # This comments
    /// # This comments
    ///
    /// key = "value"
    /// ```
    #[inline]
    pub fn dangling_comment_groups(&self) -> impl Iterator<Item = crate::DanglingCommentGroup> {
        support::comment::dangling_comment_groups(
            self.syntax()
                .child_elements()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!(']')))
                .skip_while(|node_or_token| {
                    !matches!(node_or_token.kind(), LINE_BREAK | DANGLING_COMMENT_GROUP)
                }),
        )
    }

    #[inline]
    pub fn key_value_groups(&self) -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>> {
        support::comment::dangling_comment_group_or(
            self.syntax()
                .child_elements()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!(']')))
                .skip_while(|node_or_token| {
                    !matches!(node_or_token.kind(), LINE_BREAK | DANGLING_COMMENT_GROUP)
                }),
        )
    }

    #[inline]
    pub fn key_values(&self) -> impl Iterator<Item = crate::KeyValue> {
        self.key_value_groups()
            .filter_map(DanglingCommentGroupOr::into_item_group)
            .flat_map(KeyValueGroup::into_key_values)
    }

    #[inline]
    pub fn contains_header(&self, position: tombi_text::Position) -> bool {
        self.bracket_start()
            .is_some_and(|start| start.range().end <= position)
            && self
                .bracket_end()
                .is_none_or(|end| position <= end.range().start)
    }

    /// Returns an iterator over the sub-tables of this table.
    ///
    /// ```toml
    /// [foo]  # <- This is a self table
    /// [foo.bar]  # <- This is a subtable
    /// key = "value"
    ///
    /// [[foo.bar.baz]]  # <- This is also a subtable
    /// key = true
    /// ```
    #[inline]
    pub fn sub_tables(&self) -> impl Iterator<Item = TableOrArrayOfTable> + '_ {
        support::node::next_siblings_nodes(self)
            .skip(1)
            .take_while(|t: &TableOrArrayOfTable| {
                let Some(keys) = t.header() else {
                    return false;
                };
                let Some(self_keys) = self.header() else {
                    return false;
                };

                keys.starts_with(&self_keys) && keys.keys().count() != self_keys.keys().count()
            })
    }

    #[inline]
    pub fn parent_table_or_array_of_table_keys(
        &self,
        toml_version: TomlVersion,
    ) -> impl Iterator<Item = crate::Keys> + '_ {
        support::node::prev_siblings_nodes(self)
            .filter_map(|node: TableOrArrayOfTable| node.header())
            .take_while(move |keys| {
                match (
                    self.header().and_then(|header| header.keys().next()),
                    keys.keys().next(),
                ) {
                    (Some(a), Some(b)) => match (
                        a.try_to_content(toml_version),
                        b.try_to_content(toml_version),
                    ) {
                        (Ok(a), Ok(b)) => a == b,
                        _ => false,
                    },
                    _ => false,
                }
            })
            .filter(|keys| {
                self.header()
                    .map(|header_keys| header_keys.starts_with(keys))
                    .unwrap_or_default()
            })
    }

    #[inline]
    pub fn parent_array_of_tables_keys(
        &self,
        toml_version: TomlVersion,
    ) -> impl Iterator<Item = crate::Keys> + '_ {
        support::node::prev_siblings_nodes(self)
            .filter_map(|node: ArrayOfTable| node.header())
            .take_while(move |keys| {
                match (
                    self.header().and_then(|header| header.keys().next()),
                    keys.keys().next(),
                ) {
                    (Some(a), Some(b)) => match (
                        a.try_to_content(toml_version),
                        b.try_to_content(toml_version),
                    ) {
                        (Ok(a), Ok(b)) => a == b,
                        _ => false,
                    },
                    _ => false,
                }
            })
            .filter(|keys| {
                self.header()
                    .map(|header_keys| header_keys.starts_with(keys))
                    .unwrap_or_default()
            })
    }
}
