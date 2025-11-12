use tombi_syntax::{SyntaxKind::*, T};
use tombi_toml_version::TomlVersion;

use crate::{
    support, ArrayOfTable, AstChildren, AstNode, TableOrArrayOfTable, TombiValueCommentDirective,
};

impl crate::Table {
    /// The leading comments of the table header.
    ///
    /// ```toml
    /// # This comment
    /// [table]
    /// ```
    pub fn header_leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        support::node::leading_comments(self.syntax().children_with_tokens())
    }

    /// The trailing comment of the table header.
    ///
    /// ```toml
    /// [table]  # This comment
    /// ```
    pub fn header_trailing_comment(&self) -> Option<crate::TrailingComment> {
        support::node::trailing_comment(self.syntax().children_with_tokens(), T!(']'))
    }

    /// The dangling comments of the table (without key-value pairs).
    ///
    /// ```toml
    /// [table]
    /// # This comments
    /// # This comments
    /// ```
    pub fn key_values_dangling_comments(&self) -> Vec<Vec<crate::DanglingComment>> {
        support::node::dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| !matches!(node.kind(), T!(']')))
                .skip_while(|node| !matches!(node.kind(), LINE_BREAK))
                .take_while(|node| matches!(node.kind(), COMMENT | LINE_BREAK | WHITESPACE)),
        )
    }

    /// The begin dangling comments of the table.
    ///
    /// ```toml
    /// [table]
    /// # This comments
    /// # This comments
    /// key = "value"
    /// ```
    pub fn key_values_begin_dangling_comments(&self) -> Vec<Vec<crate::BeginDanglingComment>> {
        support::node::begin_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| !matches!(node.kind(), T!(']')))
                .skip_while(|node| !matches!(node.kind(), LINE_BREAK))
                .take_while(|node| matches!(node.kind(), COMMENT | LINE_BREAK | WHITESPACE)),
        )
    }

    /// The end dangling comments of the table.
    ///
    /// ```toml
    /// [table]
    /// key = "value"
    /// # This comments
    /// # This comments
    /// ```
    pub fn key_values_end_dangling_comments(&self) -> Vec<Vec<crate::EndDanglingComment>> {
        support::node::end_dangling_comments(self.syntax().children_with_tokens())
    }

    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        itertools::chain!(
            self.header_comment_directives(),
            self.key_values_comment_directives(),
        )
    }

    pub fn header_comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        let mut header_comment_directives = vec![];

        for comment in self.header_leading_comments() {
            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                header_comment_directives.push(comment_directive);
            }
        }
        if let Some(comment) = self.header_trailing_comment() {
            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                header_comment_directives.push(comment_directive);
            }
        }

        header_comment_directives.into_iter()
    }

    pub fn key_values_comment_directives(
        &self,
    ) -> impl Iterator<Item = TombiValueCommentDirective> {
        let mut key_values_comment_directives = vec![];
        if self.key_values().next().is_none() {
            for comments in self.key_values_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        key_values_comment_directives.push(comment_directive);
                    }
                }
            }
        } else {
            for comments in self.key_values_begin_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        key_values_comment_directives.push(comment_directive);
                    }
                }
            }
            for comments in self.key_values_end_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        key_values_comment_directives.push(comment_directive);
                    }
                }
            }
        }

        key_values_comment_directives.into_iter()
    }

    pub fn contains_header(&self, position: tombi_text::Position) -> bool {
        self.bracket_start().unwrap().range().end <= position
            && position <= self.bracket_end().unwrap().range().start
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

    pub fn parent_array_of_tables_keys(
        &self,
        toml_version: TomlVersion,
    ) -> impl Iterator<Item = AstChildren<crate::Key>> + '_ {
        support::node::prev_siblings_nodes(self)
            .filter_map(|node: ArrayOfTable| node.header().map(|header| header.keys()))
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
}
