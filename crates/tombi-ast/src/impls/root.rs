use itertools::Itertools;
use tombi_syntax::SyntaxKind;

use crate::{support, AstNode};

impl crate::Root {
    pub fn schema_directive(
        &self,
        source_path: Option<&std::path::Path>,
    ) -> Option<(Result<url::Url, String>, tombi_text::Range)> {
        if let Some(comments) = self.get_document_header_comments() {
            for comment in comments {
                if let Some((schema_url, scheme_range)) = comment.schema_directive(source_path) {
                    return Some((schema_url, scheme_range));
                }
            }
        }
        None
    }

    pub fn tombi_directives(
        &self,
    ) -> Option<Vec<((String, tombi_text::Range), tombi_text::Range)>> {
        let mut tombi_directives = vec![];
        if let Some(comments) = self.get_document_header_comments() {
            for comment in comments {
                if let Some((tombi_directive, scheme_range)) = comment.tombi_directive() {
                    tombi_directives.push((tombi_directive, scheme_range));
                }
            }
        }

        if tombi_directives.is_empty() {
            None
        } else {
            Some(tombi_directives)
        }
    }

    #[inline]
    pub fn get_document_header_comments(&self) -> Option<Vec<crate::Comment>> {
        itertools::chain!(
            self.key_values_begin_dangling_comments()
                .into_iter()
                .next()
                .map(|comment| { comment.into_iter().map(crate::Comment::from).collect_vec() }),
            self.key_values_dangling_comments()
                .into_iter()
                .next()
                .map(|comment| { comment.into_iter().map(crate::Comment::from).collect_vec() }),
            self.items().next().map(|item| {
                item.leading_comments()
                    .map(crate::Comment::from)
                    .collect_vec()
            }),
        )
        .find(|comments| !comments.is_empty())
    }

    #[inline]
    pub fn key_values(&self) -> impl Iterator<Item = crate::KeyValue> {
        self.items().filter_map(|item| match item {
            crate::RootItem::KeyValue(key_value) => Some(key_value),
            _ => None,
        })
    }

    #[inline]
    pub fn table_or_array_of_tables(&self) -> impl Iterator<Item = crate::TableOrArrayOfTable> {
        self.items().filter_map(|item| match item {
            crate::RootItem::Table(table) => Some(crate::TableOrArrayOfTable::Table(table)),
            crate::RootItem::ArrayOfTable(array_of_table) => {
                Some(crate::TableOrArrayOfTable::ArrayOfTable(array_of_table))
            }
            _ => None,
        })
    }

    pub fn key_values_begin_dangling_comments(&self) -> Vec<Vec<crate::BeginDanglingComment>> {
        support::node::begin_dangling_comments(self.syntax().children_with_tokens())
    }

    pub fn key_values_end_dangling_comments(&self) -> Vec<Vec<crate::EndDanglingComment>> {
        support::node::end_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .take_while(|node_or_token| node_or_token.kind() != SyntaxKind::TABLE),
        )
    }

    pub fn key_values_dangling_comments(&self) -> Vec<Vec<crate::DanglingComment>> {
        support::node::dangling_comments(self.syntax().children_with_tokens())
    }
}
