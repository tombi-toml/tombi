use tombi_ast::TombiValueCommentDirective;

use crate::{
    support::chrono::try_new_local_time, value::collect_comment_directives_and_errors,
    DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LocalTime {
    value: tombi_date_time::LocalTime,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
}

impl LocalTime {
    #[inline]
    pub fn value(&self) -> &tombi_date_time::LocalTime {
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

impl std::fmt::Display for LocalTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ValueImpl for LocalTime {
    fn value_type(&self) -> ValueType {
        ValueType::LocalTime
    }

    fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl From<crate::LocalTime> for tombi_date_time::LocalTime {
    fn from(node: crate::LocalTime) -> Self {
        node.value
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::LocalTime {
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

        match try_new_local_time(&self, toml_version) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::LocalTime(crate::LocalTime {
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
