use tombi_ast::TombiValueCommentDirective;
use tombi_toml_version::TomlVersion;

use crate::{
    support::integer::{try_from_binary, try_from_decimal, try_from_hexadecimal, try_from_octal},
    value::collect_comment_directives_and_errors,
    DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerKind {
    Binary,
    Decimal,
    Octal,
    Hexadecimal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    kind: IntegerKind,
    value: i64,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl Integer {
    #[inline]
    pub fn kind(&self) -> IntegerKind {
        self.kind
    }

    #[inline]
    pub fn value(&self) -> i64 {
        self.value
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.range
    }

    #[inline]
    pub fn symbol_range(&self) -> tombi_text::Range {
        self.range
    }

    #[inline]
    pub fn comment_directives(&self) -> Option<&[TombiValueCommentDirective]> {
        self.comment_directives.as_deref()
    }
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ValueImpl for Integer {
    fn value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::IntegerBin {
    fn into_document_tree_and_errors(
        self,
        _toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let (comment_directives, mut errors) = collect_comment_directives_and_errors(&self);

        let Some(token) = self.token() else {
            errors.push(crate::Error::IncompleteNode { range });

            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors,
            };
        };

        match try_from_binary(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Integer(crate::Integer {
                    kind: IntegerKind::Binary,
                    value,
                    range: token.range(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseIntError { error, range });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { range },
                    errors,
                }
            }
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::IntegerOct {
    fn into_document_tree_and_errors(
        self,
        _toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let (comment_directives, mut errors) = collect_comment_directives_and_errors(&self);

        let Some(token) = self.token() else {
            errors.push(crate::Error::IncompleteNode { range });

            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors,
            };
        };

        match try_from_octal(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Integer(crate::Integer {
                    kind: IntegerKind::Octal,
                    value,
                    range: token.range(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseIntError { error, range });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { range },
                    errors,
                }
            }
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::IntegerDec {
    fn into_document_tree_and_errors(
        self,
        _toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let (comment_directives, mut errors) = collect_comment_directives_and_errors(&self);

        let Some(token) = self.token() else {
            errors.push(crate::Error::IncompleteNode { range });

            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors,
            };
        };

        match try_from_decimal(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Integer(crate::Integer {
                    kind: IntegerKind::Decimal,
                    value,
                    range: token.range(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseIntError { error, range });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { range },
                    errors,
                }
            }
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::IntegerHex {
    fn into_document_tree_and_errors(
        self,
        _toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let (comment_directives, mut errors) = collect_comment_directives_and_errors(&self);

        let Some(token) = self.token() else {
            errors.push(crate::Error::IncompleteNode { range });

            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors,
            };
        };

        match try_from_hexadecimal(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Integer(crate::Integer {
                    kind: IntegerKind::Hexadecimal,
                    value,
                    range: token.range(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseIntError { error, range });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { range },
                    errors,
                }
            }
        }
    }
}
