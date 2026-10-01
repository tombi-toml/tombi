use tombi_ast_syntax::TombiValueCommentDirective;

use crate::{
    DocumentTreeAndErrors, IntoDocumentTreeWithContext, ValueImpl, ValueType,
    support::float::try_from_float, value::collect_comment_directives_and_errors,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    value: f64,
    span: tombi_text::Span,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl Float {
    #[inline]
    pub fn value(&self) -> f64 {
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

impl std::fmt::Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ValueImpl for Float {
    fn value_type(&self) -> ValueType {
        ValueType::Float
    }

    fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::Float<'t> {
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

        match try_from_float(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Float(crate::Float {
                    value,
                    span: token.span(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseFloatError { error, span });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { span },
                    errors,
                }
            }
        }
    }
}
