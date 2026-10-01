use std::{ops::Deref, sync::Arc};

use crate::{DocumentTreeAndErrors, IntoDocumentTreeWithContext, Table};

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentTree<'t> {
    table: Arc<Table<'t>>,
}

impl<'t> From<DocumentTree<'t>> for Table<'t> {
    fn from(tree: DocumentTree<'t>) -> Self {
        Arc::try_unwrap(tree.table).unwrap_or_else(|table| (*table).clone())
    }
}

impl<'t> From<DocumentTree<'t>> for crate::Value<'t> {
    fn from(tree: DocumentTree<'t>) -> Self {
        crate::Value::Table(tree.into())
    }
}

impl<'t> Deref for DocumentTree<'t> {
    type Target = Table<'t>;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::DocumentTree<'t>> for tombi_ast_syntax::Root<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> crate::DocumentTreeAndErrors<crate::DocumentTree<'t>> {
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

        for table_or_array_of_table in self.table_or_array_of_tables() {
            let (table, errs) = match table_or_array_of_table {
                tombi_ast_syntax::TableOrArrayOfTable::Table(table) => {
                    table.into_document_tree_with_context(context)
                }
                tombi_ast_syntax::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                    array_of_table.into_document_tree_with_context(context)
                }
            }
            .into();

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
