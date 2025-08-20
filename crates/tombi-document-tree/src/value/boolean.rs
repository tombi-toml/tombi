use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_toml_version::TomlVersion;

use crate::{Comment, DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boolean {
    value: bool,
    node: tombi_ast::Boolean,
    leading_comments: Vec<Comment>,
    trailing_comment: Option<Comment>,
}

impl Boolean {
    #[inline]
    pub fn value(&self) -> bool {
        self.value
    }

    #[inline]
    pub fn node(&self) -> &tombi_ast::Boolean {
        &self.node
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.node.token().unwrap().range()
    }

    #[inline]
    pub fn symbol_range(&self) -> tombi_text::Range {
        self.range()
    }

    #[inline]
    pub fn leading_comments(&self) -> &[Comment] {
        self.leading_comments.as_ref()
    }

    #[inline]
    pub fn trailing_comment(&self) -> Option<&Comment> {
        self.trailing_comment.as_ref()
    }
}

impl ValueImpl for Boolean {
    fn value_type(&self) -> ValueType {
        ValueType::Boolean
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::Boolean {
    fn into_document_tree_and_errors(
        self,
        _toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        let value = match token.text() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        };

        let leading_comments = self.leading_comments().map(Comment::from).collect_vec();
        let trailing_comment = self.trailing_comment().map(Comment::from);

        DocumentTreeAndErrors {
            tree: crate::Value::Boolean(crate::Boolean {
                value,
                node: self,
                leading_comments,
                trailing_comment,
            }),
            errors: Vec::with_capacity(0),
        }
    }
}
