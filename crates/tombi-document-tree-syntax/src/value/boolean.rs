use tombi_ast_syntax::TombiValueCommentDirective;

use crate::{
    DocumentTreeAndErrors, IntoDocumentTreeWithContext, ValueImpl, ValueType,
    value::collect_comment_directives_and_errors,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean {
    value: bool,
    span: tombi_text::Span,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl std::fmt::Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Boolean {
    #[inline]
    pub fn value(&self) -> bool {
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

impl ValueImpl for Boolean {
    fn value_type(&self) -> ValueType {
        ValueType::Boolean
    }

    fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::Boolean<'t> {
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

        let value = match token.text() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        };

        DocumentTreeAndErrors {
            tree: crate::Value::Boolean(crate::Boolean {
                value,
                span: token.span(),
                comment_directives,
            }),
            errors,
        }
    }
}
