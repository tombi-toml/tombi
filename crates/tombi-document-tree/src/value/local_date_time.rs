use tombi_ast::TombiValueCommentDirective;

use crate::{
    support::chrono::try_new_local_date_time, value::collect_comment_directives_and_errors,
    DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDateTime {
    value: tombi_date_time::LocalDateTime,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
}

impl LocalDateTime {
    #[inline]
    pub fn value(&self) -> &tombi_date_time::LocalDateTime {
        &self.value
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
        self.comment_directives.as_deref().map(|v| &**v)
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

    fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl From<crate::LocalDateTime> for tombi_date_time::LocalDateTime {
    fn from(node: crate::LocalDateTime) -> Self {
        node.value
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::LocalDateTime {
    fn into_document_tree_and_errors(
        self,
        toml_version: tombi_toml_version::TomlVersion,
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

        match try_new_local_date_time(&self, toml_version) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::LocalDateTime(crate::LocalDateTime {
                    value,
                    range: token.range(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(error);

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { range },
                    errors,
                }
            }
        }
    }
}
