use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_toml_version::TomlVersion;

use crate::{
    support::float::try_from_float, Comment, DocumentTreeAndErrors, IntoDocumentTreeAndErrors,
    ValueImpl, ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    value: f64,
    node: tombi_ast::Float,
    leading_comments: Vec<Comment>,
    trailing_comment: Option<Comment>,
}

impl Float {
    #[inline]
    pub fn value(&self) -> f64 {
        self.value
    }

    #[inline]
    pub fn node(&self) -> &tombi_ast::Float {
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

impl ValueImpl for Float {
    fn value_type(&self) -> ValueType {
        ValueType::Float
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::Float {
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

        match try_from_float(token.text()) {
            Ok(value) => {
                let leading_comments = self.leading_comments().map(Comment::from).collect_vec();
                let trailing_comment = self.trailing_comment().map(Comment::from);

                DocumentTreeAndErrors {
                    tree: crate::Value::Float(crate::Float {
                        value,
                        node: self,
                        leading_comments,
                        trailing_comment,
                    }),
                    errors: Vec::with_capacity(0),
                }
            }
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseFloatError { error, range }],
            },
        }
    }
}
