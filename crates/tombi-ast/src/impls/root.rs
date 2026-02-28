use itertools::Itertools;

use crate::{
    AstNode, DanglingCommentGroupOr, KeyValueGroup, SchemaDocumentCommentDirective,
    TombiDocumentCommentDirective, TombiValueCommentDirective,
    comment_directive::DocumentCommentDirectives, support,
};

impl crate::Root {
    /// Returns the leading comments of the first item (key-value or table/array-of-table).
    pub fn first_item_leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        if let Some(first_key_value) = self.key_values().next() {
            first_key_value.leading_comments().collect()
        } else if let Some(first_table_or_aot) = self.table_or_array_of_tables().next() {
            first_table_or_aot.leading_comments().collect()
        } else {
            Vec::with_capacity(0)
        }
        .into_iter()
    }

    pub fn document_comment_directive(
        &self,
        source_path: Option<&std::path::Path>,
    ) -> Option<DocumentCommentDirectives> {
        DocumentCommentDirectives::from_comments(
            self.dangling_comment_groups()
                .flat_map(|comment_group| comment_group.into_comments().map(Into::into)),
            source_path,
        )
        .or_else(|| {
            DocumentCommentDirectives::from_comments(
                self.first_item_leading_comments()
                    .into_iter()
                    .map(Into::into),
                source_path,
            )
        })
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
                    .into_iter()
                    .filter_map(|comment| comment.get_tombi_document_directive()),
            );
        }

        directives.into_iter()
    }

    pub fn comment_directives(&self) -> impl Iterator<Item = TombiValueCommentDirective> {
        self.dangling_comment_groups()
            .into_iter()
            .flat_map(|comment_group| {
                comment_group
                    .into_comments()
                    .into_iter()
                    .filter_map(|comment| comment.get_tombi_value_directive())
            })
    }

    pub fn dangling_comment_groups(&self) -> impl Iterator<Item = crate::DanglingCommentGroup> {
        support::comment::dangling_comment_groups(self.syntax().children_with_tokens())
    }

    pub fn key_value_groups(&self) -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>> {
        support::comment::dangling_comment_group_or(self.syntax().children_with_tokens())
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

    #[inline]
    pub fn table_or_array_of_tables(&self) -> impl Iterator<Item = crate::TableOrArrayOfTable> {
        self.syntax()
            .children()
            .filter_map(crate::TableOrArrayOfTable::cast)
    }
}
