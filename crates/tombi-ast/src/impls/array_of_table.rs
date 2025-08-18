use tombi_syntax::{SyntaxKind::*, T};
use tombi_toml_version::TomlVersion;

use crate::{support, ArrayOfTable, AstChildren, AstNode, TableOrArrayOfTable};

impl crate::ArrayOfTable {
    pub fn header_leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        support::node::leading_comments(self.syntax().children_with_tokens())
    }

    pub fn header_trailing_comment(&self) -> Option<crate::TrailingComment> {
        support::node::trailing_comment(self.syntax().children_with_tokens(), T!("]]"))
    }

    pub fn contains_header(&self, position: tombi_text::Position) -> bool {
        self.double_bracket_start().unwrap().range().end <= position
            && position <= self.double_bracket_end().unwrap().range().start
    }

    pub fn key_values_dangling_comments(&self) -> Vec<Vec<crate::DanglingComment>> {
        support::node::dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| !matches!(node.kind(), T!("]]")))
                .skip_while(|node| !matches!(node.kind(), LINE_BREAK))
                .take_while(|node| matches!(node.kind(), COMMENT | LINE_BREAK | WHITESPACE)),
        )
    }

    pub fn key_values_begin_dangling_comments(&self) -> Vec<Vec<crate::BeginDanglingComment>> {
        support::node::begin_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| !matches!(node.kind(), T!("]]")))
                .skip_while(|node| !matches!(node.kind(), LINE_BREAK))
                .take_while(|node| matches!(node.kind(), COMMENT | LINE_BREAK | WHITESPACE)),
        )
    }

    pub fn key_values_end_dangling_comments(&self) -> Vec<Vec<crate::EndDanglingComment>> {
        support::node::end_dangling_comments(self.syntax().children_with_tokens())
    }

    /// Returns an iterator over the subtables of this table.
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
    pub fn subtables(&self) -> impl Iterator<Item = TableOrArrayOfTable> + '_ {
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

    pub fn array_of_tables_keys(&self) -> impl Iterator<Item = AstChildren<crate::Key>> + '_ {
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
