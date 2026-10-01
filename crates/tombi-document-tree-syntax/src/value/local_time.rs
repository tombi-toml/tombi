use tombi_ast_syntax::TombiValueCommentDirective;

use crate::{
    DocumentTreeAndErrors, IntoDocumentTreeWithContext, ValueImpl, ValueType,
    support::chrono::try_new_local_time, value::collect_comment_directives_and_errors,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LocalTime {
    value: tombi_date_time::LocalTime,
    span: tombi_text::Span,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl LocalTime {
    #[inline]
    pub fn value(&self) -> &tombi_date_time::LocalTime {
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

impl std::fmt::Display for LocalTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ValueImpl for LocalTime {
    fn value_type(&self) -> ValueType {
        ValueType::LocalTime
    }

    fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl From<crate::LocalTime> for tombi_date_time::LocalTime {
    fn from(node: crate::LocalTime) -> Self {
        node.value
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::LocalTime<'t> {
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

        match try_new_local_time(&self, context.toml_version) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::LocalTime(crate::LocalTime {
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
