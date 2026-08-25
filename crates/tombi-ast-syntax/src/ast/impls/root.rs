use itertools::Itertools;

use crate::{
    AstNode, DanglingCommentGroupOr, KeyValueGroup, SchemaDocumentCommentDirective,
    TombiDocumentCommentDirective, TombiValueCommentDirective, support,
};

impl crate::Root {
    pub fn first_line_break(&self) -> Option<crate::SyntaxToken> {
        self.syntax()
            .first_token()
            .filter(|token| token.kind() == crate::SyntaxKind::LINE_BREAK)
    }

    pub fn comment_at_position(&self, position: tombi_text::Position) -> Option<crate::Comment> {
        use crate::AstToken;

        match self.syntax().token_at_position(position) {
            crate::TokenAtOffset::Single(token) => crate::Comment::cast(token),
            crate::TokenAtOffset::Between(left, right) => {
                crate::Comment::cast(left).or_else(|| crate::Comment::cast(right))
            }
            crate::TokenAtOffset::None => None,
        }
    }

    /// TOML nodes in source order. Punctuation and trivia are intentionally
    /// excluded; malformed syntax is represented by `TomlNode::Invalid`.
    pub fn nodes(&self) -> impl Iterator<Item = crate::TomlNode> {
        self.syntax()
            .descendants()
            .filter_map(crate::TomlNode::cast)
    }

    /// TOML nodes containing `position`, ordered from the innermost node to
    /// the root. Parser-only grouping nodes are intentionally omitted.
    pub fn nodes_at_position(
        &self,
        position: tombi_text::Position,
    ) -> impl Iterator<Item = crate::TomlNode> {
        crate::algo::ancestors_at_position(self.syntax(), position)
            .filter_map(crate::TomlNode::cast)
    }

    /// Returns commas immediately before and after the syntax item at
    /// `position`, including commas recovered as invalid syntax.
    pub fn adjacent_commas(&self, position: tombi_text::Position) -> crate::AdjacentCommas {
        use crate::{Direction, NodeOrToken, SyntaxElement, SyntaxKind};

        let Some(node) = crate::algo::ancestors_at_position(self.syntax(), position).next() else {
            return crate::AdjacentCommas::default();
        };

        let before = node
            .last_child()
            .filter(|child| child.kind() == SyntaxKind::COMMA)
            .map(|child| child.range())
            .or_else(|| {
                node.siblings_with_tokens(Direction::Prev)
                    .find(|element| !element.range().contains(position))
                    .filter(|element| element.kind() == SyntaxKind::COMMA)
                    .map(|element| element.range())
            });

        let after = node
            .siblings_with_tokens(Direction::Next)
            .next()
            .and_then(|element| match element.kind() {
                SyntaxKind::COMMA => Some(element.range()),
                SyntaxKind::INVALID_TOKEN => match element {
                    NodeOrToken::Node(node) => node
                        .first_child_or_token()
                        .and_then(SyntaxElement::into_token)
                        .filter(|token| token.kind() == SyntaxKind::COMMA)
                        .map(|token| token.range()),
                    NodeOrToken::Token(_) => None,
                },
                SyntaxKind::ARRAY => match element {
                    NodeOrToken::Node(node) => crate::Array::cast(node)
                        .and_then(|array| array.comma_after(position))
                        .map(|comma| comma.range()),
                    NodeOrToken::Token(_) => None,
                },
                _ => None,
            });

        crate::AdjacentCommas { before, after }
    }

    /// Innermost key-value containing `position`.
    pub fn enclosing_key_value(&self, position: tombi_text::Position) -> Option<crate::KeyValue> {
        self.nodes_at_position(position)
            .find_map(|node| match node {
                crate::TomlNode::KeyValue(key_value) => Some(key_value),
                _ => None,
            })
    }

    pub fn array_at_range(&self, range: tombi_text::Range) -> Option<crate::Array> {
        self.nodes().find_map(|node| match node {
            crate::TomlNode::Array(array) if array.range() == range => Some(array),
            _ => None,
        })
    }

    pub fn inline_table_at_range(&self, range: tombi_text::Range) -> Option<crate::InlineTable> {
        self.nodes().find_map(|node| match node {
            crate::TomlNode::InlineTable(table) if table.range() == range => Some(table),
            _ => None,
        })
    }

    /// Returns the leading comments of the first item (key-value or table/array-of-table).
    pub fn first_item_leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        if let Some(first_key_value) = self.key_values().next() {
            first_key_value.leading_comments().collect()
        } else if let Some(first_table_or_aot) = self.table_or_array_of_tables().next() {
            first_table_or_aot.leading_comments().collect()
        } else {
            Vec::new()
        }
        .into_iter()
    }

    pub fn schema_document_comment_directive(
        &self,
        source_path: Option<&std::path::Path>,
    ) -> Option<SchemaDocumentCommentDirective> {
        for comment_group in self.dangling_comment_groups() {
            for comment in comment_group.comments() {
                if let Some(schema_directive) = comment.get_document_schema_directive(source_path) {
                    return Some(schema_directive);
                }
            }
        }

        for comment in self.first_item_leading_comments() {
            if let Some(schema_directive) = comment.get_document_schema_directive(source_path) {
                return Some(schema_directive);
            }
        }

        None
    }

    pub fn tombi_document_comment_directives(
        &self,
    ) -> impl Iterator<Item = TombiDocumentCommentDirective> {
        let mut directives = self
            .dangling_comment_groups()
            .flat_map(|comment_group| {
                comment_group
                    .into_comments()
                    .filter_map(|comment| comment.get_tombi_document_directive())
            })
            .collect_vec();

        if directives.is_empty() {
            directives.extend(
                self.first_item_leading_comments()
                    .filter_map(|comment| comment.get_tombi_document_directive()),
            );
        }

        directives.into_iter()
    }

    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        self.dangling_comment_groups().flat_map(|comment_group| {
            comment_group
                .into_comments()
                .filter_map(|comment| comment.get_tombi_value_directive())
        })
    }

    pub fn dangling_comment_groups(&self) -> impl Iterator<Item = crate::DanglingCommentGroup> {
        support::comment::dangling_comment_groups(self.syntax().child_elements())
    }

    pub fn key_value_groups(&self) -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>> {
        support::comment::dangling_comment_group_or(self.syntax().child_elements())
    }

    pub fn key_values(&self) -> impl Iterator<Item = crate::KeyValue> {
        self.key_value_groups()
            .filter_map(|group| {
                group
                    .into_item_group()
                    .map(|key_value_group| key_value_group.into_key_values())
            })
            .flatten()
    }

    pub fn items(&self) -> impl Iterator<Item = crate::RootItem> {
        self.key_values()
            .map(crate::RootItem::from)
            .chain(self.table_or_array_of_tables().map(|item| match item {
                crate::TableOrArrayOfTable::Table(table) => crate::RootItem::Table(table),
                crate::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                    crate::RootItem::ArrayOfTable(array_of_table)
                }
            }))
    }

    #[inline]
    pub fn table_or_array_of_tables(&self) -> impl Iterator<Item = crate::TableOrArrayOfTable> {
        self.syntax()
            .child_nodes()
            .filter_map(crate::TableOrArrayOfTable::cast)
    }
}
