use tombi_ast::TombiValueCommentDirective;

use crate::{
    support::chrono::try_new_local_date, DocumentTreeAndErrors, IntoDocumentTreeAndErrors,
    ValueImpl, ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDate {
    value: tombi_date_time::LocalDate,
    range: tombi_text::Range,
    comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
}

impl LocalDate {
    #[inline]
    pub fn value(&self) -> &tombi_date_time::LocalDate {
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

impl ValueImpl for LocalDate {
    fn value_type(&self) -> ValueType {
        ValueType::LocalDate
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl From<crate::LocalDate> for tombi_date_time::LocalDate {
    fn from(node: crate::LocalDate) -> Self {
        node.value
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::LocalDate {
    fn into_document_tree_and_errors(
        self,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        match try_new_local_date(&self, toml_version) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::LocalDate(crate::LocalDate {
                    value,
                    range: token.range(),
                    comment_directives: None,
                }),
                errors: Vec::with_capacity(0),
            },
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![error],
            },
        }
    }
}