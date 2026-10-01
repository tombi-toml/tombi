use tombi_ast_syntax::TombiValueCommentDirective;

use crate::{
    DocumentTreeAndErrors, IntoDocumentTreeWithContext, ValueImpl, ValueType,
    support::integer::{try_from_binary, try_from_decimal, try_from_hexadecimal, try_from_octal},
    value::collect_comment_directives_and_errors,
};

use tombi_document_tree::IntegerKind;

#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    kind: IntegerKind,
    value: i64,
    span: tombi_text::Span,
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
    pub fn span(&self) -> tombi_text::Span {
        self.span
    }

    #[inline]
    pub fn comment_directives(
        &self,
    ) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.comment_directives.as_deref().map(|d| d.iter())
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

    fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::IntegerBin<'t> {
    fn into_document_tree_with_context(
        self,
        _context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let span = self.span();
        let (comment_directives, mut errors) = collect_comment_directives_and_errors(&self);

        let Some(token) = self.token() else {
            errors.push(crate::Error::IncompleteNode { span });

            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { span },
                errors,
            };
        };

        match try_from_binary(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Integer(crate::Integer {
                    kind: IntegerKind::Binary,
                    value,
                    span: token.span(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseIntError { error, span });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { span },
                    errors,
                }
            }
        }
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::IntegerOct<'t> {
    fn into_document_tree_with_context(
        self,
        _context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let span = self.span();
        let (comment_directives, mut errors) = collect_comment_directives_and_errors(&self);

        let Some(token) = self.token() else {
            errors.push(crate::Error::IncompleteNode { span });

            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { span },
                errors,
            };
        };

        match try_from_octal(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Integer(crate::Integer {
                    kind: IntegerKind::Octal,
                    value,
                    span: token.span(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseIntError { error, span });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { span },
                    errors,
                }
            }
        }
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::IntegerDec<'t> {
    fn into_document_tree_with_context(
        self,
        _context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let span = self.span();
        let (comment_directives, mut errors) = collect_comment_directives_and_errors(&self);

        let Some(token) = self.token() else {
            errors.push(crate::Error::IncompleteNode { span });

            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { span },
                errors,
            };
        };

        match try_from_decimal(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Integer(crate::Integer {
                    kind: IntegerKind::Decimal,
                    value,
                    span: token.span(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseIntError { error, span });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { span },
                    errors,
                }
            }
        }
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::IntegerHex<'t> {
    fn into_document_tree_with_context(
        self,
        _context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let span = self.span();
        let (comment_directives, mut errors) = collect_comment_directives_and_errors(&self);

        let Some(token) = self.token() else {
            errors.push(crate::Error::IncompleteNode { span });

            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { span },
                errors,
            };
        };

        match try_from_hexadecimal(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Integer(crate::Integer {
                    kind: IntegerKind::Hexadecimal,
                    value,
                    span: token.span(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseIntError { error, span });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { span },
                    errors,
                }
            }
        }
    }
}
