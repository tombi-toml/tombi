use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_toml_version::TomlVersion;

use crate::{
    support::integer::{try_from_binary, try_from_decimal, try_from_hexadecimal, try_from_octal},
    Comment, DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerKind {
    Binary(tombi_ast::IntegerBin),
    Decimal(tombi_ast::IntegerDec),
    Octal(tombi_ast::IntegerOct),
    Hexadecimal(tombi_ast::IntegerHex),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Integer {
    kind: IntegerKind,
    value: i64,
    leading_comments: Vec<Comment>,
    trailing_comment: Option<Comment>,
}

impl Integer {
    #[inline]
    pub fn kind(&self) -> &IntegerKind {
        &self.kind
    }

    #[inline]
    pub fn value(&self) -> i64 {
        self.value
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        match self.kind() {
            IntegerKind::Binary(node) => node.token(),
            IntegerKind::Decimal(node) => node.token(),
            IntegerKind::Octal(node) => node.token(),
            IntegerKind::Hexadecimal(node) => node.token(),
        }
        .unwrap()
        .range()
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

impl ValueImpl for Integer {
    fn value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::IntegerBin {
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

        match try_from_binary(token.text()) {
            Ok(value) => {
                let leading_comments = self.leading_comments().map(Comment::from).collect_vec();
                let trailing_comment = self.trailing_comment().map(Comment::from);

                DocumentTreeAndErrors {
                    tree: crate::Value::Integer(crate::Integer {
                        kind: IntegerKind::Binary(self),
                        value,
                        leading_comments,
                        trailing_comment,
                    }),
                    errors: Vec::with_capacity(0),
                }
            }
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseIntError { error, range }],
            },
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::IntegerOct {
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

        match try_from_octal(token.text()) {
            Ok(value) => {
                let leading_comments = self.leading_comments().map(Comment::from).collect_vec();
                let trailing_comment = self.trailing_comment().map(Comment::from);

                DocumentTreeAndErrors {
                    tree: crate::Value::Integer(crate::Integer {
                        kind: IntegerKind::Octal(self),
                        value,
                        leading_comments,
                        trailing_comment,
                    }),
                    errors: Vec::with_capacity(0),
                }
            }
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseIntError { error, range }],
            },
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::IntegerDec {
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

        match try_from_decimal(token.text()) {
            Ok(value) => {
                let leading_comments = self.leading_comments().map(Comment::from).collect_vec();
                let trailing_comment = self.trailing_comment().map(Comment::from);

                DocumentTreeAndErrors {
                    tree: crate::Value::Integer(crate::Integer {
                        kind: IntegerKind::Decimal(self),
                        value,
                        leading_comments,
                        trailing_comment,
                    }),
                    errors: Vec::with_capacity(0),
                }
            }
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseIntError { error, range }],
            },
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::IntegerHex {
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

        match try_from_hexadecimal(token.text()) {
            Ok(value) => {
                let leading_comments = self.leading_comments().map(Comment::from).collect_vec();
                let trailing_comment = self.trailing_comment().map(Comment::from);

                DocumentTreeAndErrors {
                    tree: crate::Value::Integer(crate::Integer {
                        kind: IntegerKind::Hexadecimal(self),
                        value,
                        leading_comments,
                        trailing_comment,
                    }),
                    errors: Vec::with_capacity(0),
                }
            }
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseIntError { error, range }],
            },
        }
    }
}
