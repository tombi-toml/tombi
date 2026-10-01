use std::{ops::Deref, sync::Arc};

use crate::{DocumentTreeAndErrors, IntoDocumentTreeWithContext, Table};

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentTree {
    table: Arc<Table>,
    /// The line index of the source, built while parsing it.
    line_index: Arc<tombi_text::LineIndex>,
}

impl DocumentTree {
    /// The line index of the source, to convert the spans of the tree into ranges.
    #[inline]
    pub fn line_index(&self) -> &Arc<tombi_text::LineIndex> {
        &self.line_index
    }
}

impl From<DocumentTree> for Table {
    fn from(tree: DocumentTree) -> Self {
        Arc::try_unwrap(tree.table).unwrap_or_else(|table| (*table).clone())
    }
}

impl From<DocumentTree> for crate::Value {
    fn from(tree: DocumentTree) -> Self {
        crate::Value::Table(tree.into())
    }
}

impl Deref for DocumentTree {
    type Target = Table;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl IntoDocumentTreeWithContext<crate::DocumentTree> for tombi_ast_syntax::Root {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext,
    ) -> crate::DocumentTreeAndErrors<crate::DocumentTree> {
        let mut errors = vec![];

        let mut tree = {
            let mut table = crate::Table::new_root(&self);

            let mut body_comment_directives = vec![];
            for comment_group in self.dangling_comment_groups() {
                for comment in comment_group.comments() {
                    if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                        errors.push(error);
                    }
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        body_comment_directives.push(comment_directive);
                    }
                }
            }

            if !body_comment_directives.is_empty() {
                table.body_comment_directives = Some(body_comment_directives);
            }

            crate::DocumentTree {
                table: Arc::new(table),
                line_index: Arc::clone(tombi_ast_syntax::AstNode::syntax(&self).line_index()),
            }
        };

        {
            let mut group_boundary_comment_directives = Vec::new();
            for group in self.key_value_groups() {
                match group {
                    tombi_ast_syntax::DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                        for key_value in key_value_group.into_key_values() {
                            let (table, errs) =
                                key_value.into_document_tree_with_context(context).into();
                            if !errs.is_empty() {
                                errors.extend(errs);
                            }
                            if let Err(errs) = Arc::make_mut(&mut tree.table).merge(table) {
                                errors.extend(errs);
                            }
                        }
                    }
                    tombi_ast_syntax::DanglingCommentGroupOr::DanglingCommentGroup(
                        comment_group,
                    ) => {
                        for comment in comment_group.comments() {
                            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                                group_boundary_comment_directives.push(comment_directive);
                            }
                        }
                    }
                }
            }
            if !group_boundary_comment_directives.is_empty() {
                Arc::make_mut(&mut tree.table).group_boundary_comment_directives =
                    Some(group_boundary_comment_directives);
            }
        }

        let mut scope = crate::value::ArrayOfTablesKeysScope::default();
        for table_or_array_of_table in self.table_or_array_of_tables() {
            let (table, errs) = scope.convert(table_or_array_of_table, context).into();

            if !errs.is_empty() {
                errors.extend(errs);
            }

            if let Err(errs) = Arc::make_mut(&mut tree.table).merge(table) {
                errors.extend(errs);
            }
        }

        DocumentTreeAndErrors { tree, errors }
    }
}
