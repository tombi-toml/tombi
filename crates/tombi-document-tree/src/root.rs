use std::ops::Deref;

use tombi_toml_version::TomlVersion;

use crate::{DocumentTreeAndErrors, IntoDocumentTreeAndErrors, Table};

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentTree(pub(crate) Table);

impl From<DocumentTree> for Table {
    fn from(tree: DocumentTree) -> Self {
        tree.0
    }
}

impl From<DocumentTree> for crate::Value {
    fn from(tree: DocumentTree) -> Self {
        crate::Value::Table(tree.0)
    }
}

impl From<&DocumentTree> for &crate::Value {
    fn from(tree: &DocumentTree) -> Self {
        unsafe { std::mem::transmute(tree) }
    }
}

impl Deref for DocumentTree {
    type Target = Table;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoDocumentTreeAndErrors<crate::DocumentTree> for tombi_ast::Root {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> crate::DocumentTreeAndErrors<crate::DocumentTree> {
        let mut errors = vec![];

        let mut tree = {
            let mut table = crate::Table::new_root(&self);

            let mut inner_comment_directives = vec![];
            for comment_group in self.dangling_comment_groups() {
                for comment in comment_group.comments() {
                    if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                        errors.push(error);
                    }
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        inner_comment_directives.push(comment_directive);
                    }
                }
            }

            if !inner_comment_directives.is_empty() {
                table.inner_comment_directives = Some(inner_comment_directives);
            }

            crate::DocumentTree(table)
        };

        {
            let mut group_boundary_comment_directives = Vec::new();
            for group in self.key_value_groups() {
                match group {
                    tombi_ast::DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                        for key_value in key_value_group.into_key_values() {
                            let (table, errs) =
                                key_value.into_document_tree_and_errors(toml_version).into();
                            if !errs.is_empty() {
                                errors.extend(errs);
                            }
                            if let Err(errs) = tree.0.merge(table) {
                                errors.extend(errs);
                            }
                        }
                    }
                    tombi_ast::DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                        for comment in comment_group.comments() {
                            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                                group_boundary_comment_directives.push(comment_directive);
                            }
                        }
                    }
                }
            }
            if !group_boundary_comment_directives.is_empty() {
                tree.0.group_boundary_comment_directives =
                    Some(group_boundary_comment_directives);
            }
        }

        for table_or_array_of_table in self.table_or_array_of_tables() {
            let (table, errs) = match table_or_array_of_table {
                tombi_ast::TableOrArrayOfTable::Table(table) => {
                    table.into_document_tree_and_errors(toml_version)
                }
                tombi_ast::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                    array_of_table.into_document_tree_and_errors(toml_version)
                }
            }
            .into();

            if !errs.is_empty() {
                errors.extend(errs);
            }

            if let Err(errs) = tree.0.merge(table) {
                errors.extend(errs);
            }
        }

        DocumentTreeAndErrors { tree, errors }
    }
}
