use std::ops::Deref;

use itertools::Itertools;
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
        let key_values = self.key_values().collect_vec();
        let mut errors = vec![];

        let mut tree = {
            let mut table = crate::Table::new_root(&self);

            {
                let mut inner_comment_directives = vec![];
                if !matches!(self.items().next(), Some(tombi_ast::RootItem::KeyValue(_))) {
                    for comments in self.key_values_dangling_comments() {
                        for comment in comments {
                            if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                                errors.push(error);
                            }
                            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                                inner_comment_directives.push(comment_directive);
                            }
                        }
                    }
                } else {
                    for comments in self.key_values_begin_dangling_comments() {
                        for comment in comments {
                            if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                                errors.push(error);
                            }
                            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                                inner_comment_directives.push(comment_directive);
                            }
                        }
                    }
                    for comments in self.key_values_end_dangling_comments() {
                        for comment in comments {
                            if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                                errors.push(error);
                            }
                            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                                inner_comment_directives.push(comment_directive);
                            }
                        }
                    }
                }

                if !inner_comment_directives.is_empty() {
                    table.inner_comment_directives = Some(inner_comment_directives);
                }
            }

            crate::DocumentTree(table)
        };

        for key_value in key_values {
            let (table, errs) = key_value.into_document_tree_and_errors(toml_version).into();

            if !errs.is_empty() {
                errors.extend(errs);
            }
            if let Err(errs) = tree.0.merge(table) {
                errors.extend(errs);
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
