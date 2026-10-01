use tombi_ast_syntax::TombiValueCommentDirective;

use crate::{
    DocumentTreeAndErrors, IntoDocumentTreeWithContext, ValueImpl, ValueType,
    support::chrono::try_new_local_date_time, value::collect_comment_directives_and_errors,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDateTime {
    value: tombi_date_time::LocalDateTime,
    span: tombi_text::Span,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl LocalDateTime {
    #[inline]
    pub fn value(&self) -> &tombi_date_time::LocalDateTime {
        &self.value
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

impl std::fmt::Display for LocalDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ValueImpl for LocalDateTime {
    fn value_type(&self) -> ValueType {
        ValueType::LocalDateTime
    }

    fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl From<crate::LocalDateTime> for tombi_date_time::LocalDateTime {
    fn from(node: crate::LocalDateTime) -> Self {
        node.value
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::LocalDateTime<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
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

        match try_new_local_date_time(&self, context.toml_version) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::LocalDateTime(crate::LocalDateTime {
                    value,
                    span: token.span(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(error);

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { span },
                    errors,
                }
            }
        }
    }
}
