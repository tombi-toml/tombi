use tombi_ast::TombiValueCommentDirective;
use tombi_toml_version::TomlVersion;

use crate::{
    value::collect_comment_directives_and_errors, DocumentTreeAndErrors, IntoDocumentTreeAndErrors,
    ValueImpl, ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean {
    value: bool,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
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
    pub fn range(&self) -> tombi_text::Range {
        self.range
    }

    #[inline]
    pub fn symbol_range(&self) -> tombi_text::Range {
        self.range()
    }

    #[inline]
    pub fn comment_directives(&self) -> Option<&[TombiValueCommentDirective]> {
        self.comment_directives.as_deref().map(|v| &**v)
    }
}

impl ValueImpl for Boolean {
    fn value_type(&self) -> ValueType {
        ValueType::Boolean
    }

    fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::Boolean {
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

        let value = match token.text() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        };

        DocumentTreeAndErrors {
            tree: crate::Value::Boolean(crate::Boolean {
                value,
                range: token.range(),
                comment_directives,
            }),
            errors,
        }
    }
}
