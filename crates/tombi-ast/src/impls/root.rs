use tombi_syntax::SyntaxKind::*;

use crate::{
    AstNode, DanglingCommentGroupOr, KeyValueGroup, SchemaDocumentCommentDirective,
    TombiDocumentCommentDirective, TombiValueCommentDirective, support,
};

impl crate::Root {
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
        None
    }

    pub fn tombi_document_comment_directives(&self) -> Vec<TombiDocumentCommentDirective> {
        let mut tombi_directives = vec![];
        for comment_group in self.dangling_comment_groups() {
            for comment in comment_group.comments() {
                if let Some(tombi_directive) = comment.get_tombi_document_directive() {
                    tombi_directives.push(tombi_directive);
                }
            }
        }

        tombi_directives
    }

    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        self.dangling_comment_groups().flat_map(|comment_group| {
            comment_group
                .into_comments()
                .filter_map(|comment| comment.get_tombi_value_directive())
        })
    }

    pub fn dangling_comment_groups(&self) -> impl Iterator<Item = crate::DanglingCommentGroup> {
        support::comment::dangling_comment_groups(self.syntax().children_with_tokens())
    }

    pub fn key_value_groups(&self) -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>> {
        support::comment::dangling_comment_group_or(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node_or_token| {
                    !matches!(node_or_token.kind(), LINE_BREAK | DANGLING_COMMENT_GROUP)
                }),
        )
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
}
