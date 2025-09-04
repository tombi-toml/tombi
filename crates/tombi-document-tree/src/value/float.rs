use tombi_ast::TombiValueCommentDirective;
use tombi_toml_version::TomlVersion;

use crate::{
    support::float::try_from_float, value::collect_comment_directives_and_errors,
    DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    value: f64,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
}

impl Float {
    #[inline]
    pub fn value(&self) -> f64 {
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
        self.comment_directives.as_deref().map(|v| &**v)
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

    fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::Float {
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

        match try_from_float(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Float(crate::Float {
                    value,
                    range: token.range(),
                    comment_directives,
                }),
                errors,
            },
            Err(error) => {
                errors.push(crate::Error::ParseFloatError { error, range });

                DocumentTreeAndErrors {
                    tree: crate::Value::Incomplete { range },
                    errors,
                }
            }
        }
    }
}
