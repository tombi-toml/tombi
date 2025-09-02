use itertools::Itertools;
use tombi_syntax::SyntaxKind;

use crate::{
    support, AstNode, SchemaDocumentCommentDirective, TombiDocumentCommentDirective,
    TombiValueCommentDirective,
};

impl crate::Root {
    pub fn schema_document_comment_directive(
        &self,
        source_path: Option<&std::path::Path>,
    ) -> Option<SchemaDocumentCommentDirective> {
        if let Some(comments) = self.get_document_header_comments() {
            for comment in comments {
                if let Some(schema_directive) = comment.get_document_schema_directive(source_path) {
                    return Some(schema_directive);
                }
            }
        }
        None
    }

    pub fn tombi_document_comment_directives(&self) -> Vec<TombiDocumentCommentDirective> {
        let mut tombi_directives = vec![];
        if let Some(comments) = self.get_document_header_comments() {
            for comment in comments {
                if let Some(tombi_directive) = comment.get_tombi_document_directive() {
                    tombi_directives.push(tombi_directive);
                }
            }
        }

        tombi_directives
    }

    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        let mut inner_comment_directives = vec![];
        if self.items().next().is_none() {
            for comments in self.key_values_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        inner_comment_directives.push(comment_directive);
                    }
                }
            }
        } else {
            for comments in self.key_values_begin_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        inner_comment_directives.push(comment_directive);
                    }
                }
            }
            for comments in self.key_values_end_dangling_comments() {
                for comment in comments {
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        inner_comment_directives.push(comment_directive);
                    }
                }
            }
        }

        inner_comment_directives.into_iter()
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
