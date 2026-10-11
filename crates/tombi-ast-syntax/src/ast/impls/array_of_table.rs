use tombi_ast_syntax::{SyntaxKind::*, T};

use crate::{
    AstNode, DanglingCommentGroupOr, KeyValueGroup, TableOrArrayOfTable,
    TombiValueCommentDirective, support,
};

impl<'t> crate::ArrayOfTable<'t> {
    /// Span from the opening double bracket through the last non-trivia
    /// element owned directly by this array-of-table.
    pub fn content_span(&self) -> Option<tombi_text::Span> {
        let mut elements = self.syntax().child_elements();
        let first = elements.find(|element| element.kind() == T!("[["))?;
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
        Some(tombi_text::Span::new(first.span().start, last.span().end))
    }

    #[inline]
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
    #[inline]
    pub fn header_leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment<'t>> {
        support::comment::leading_comments(self.syntax().child_elements())
    }

    /// The trailing comment of the array of table header.
    ///
    /// ```toml
    /// [[table]]  # This comment
    /// ```
    #[inline]
    pub fn header_trailing_comment(&self) -> Option<crate::TrailingComment<'t>> {
        support::comment::trailing_comment(self.syntax().child_elements(), T!("]]"))
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
    #[inline]
    pub fn dangling_comment_groups(&self) -> impl Iterator<Item = crate::DanglingCommentGroup<'t>> {
        support::comment::dangling_comment_groups(
            self.syntax()
                .child_elements()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!("]]")))
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), LINE_BREAK)),
        )
    }

    pub fn key_value_groups(
        &self,
    ) -> impl Iterator<Item = DanglingCommentGroupOr<'t, KeyValueGroup<'t>>> {
        support::comment::dangling_comment_group_or(
            self.syntax()
                .child_elements()
                .skip_while(|node_or_token| !matches!(node_or_token.kind(), T!("]]")))
                .skip_while(|node_or_token| {
                    !matches!(node_or_token.kind(), LINE_BREAK | DANGLING_COMMENT_GROUP)
                }),
        )
    }

    #[inline]
    pub fn key_values(&self) -> impl Iterator<Item = crate::KeyValue<'t>> {
        self.key_value_groups()
            .filter_map(DanglingCommentGroupOr::into_item_group)
            .flat_map(KeyValueGroup::into_key_values)
    }

    #[inline]
    pub fn contains_header(&self, offset: tombi_text::Offset) -> bool {
        self.double_bracket_start()
            .is_some_and(|start| start.span().end <= offset)
            && self
                .double_bracket_end()
                .is_none_or(|end| offset <= end.span().start)
    }

    /// Returns the last of the sub-tables of this table.
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
    #[inline]
    pub fn last_sub_table(&self) -> Option<TableOrArrayOfTable<'t>> {
        let id = crate::header_info(self.syntax()).last_sub_table?;
        TableOrArrayOfTable::cast(self.syntax().node_at(id))
    }

    /// The number of distinct shorter key-prefixes of the header that were already declared by a
    /// preceding `[table]` / `[[array_of_tables]]` header.
    #[inline]
    pub fn parent_table_or_array_of_table_count(&self) -> usize {
        crate::header_info(self.syntax()).parent_header_count
    }

    /// For each key-prefix of the header (prefix length `i + 1` at index `i`), the number of
    /// preceding `[[array_of_tables]]` headers that equal that prefix.
    #[inline]
    pub fn parent_array_of_tables_prefix_counts(&self) -> Vec<usize> {
        crate::header_info(self.syntax()).array_of_tables_counts
    }
}
