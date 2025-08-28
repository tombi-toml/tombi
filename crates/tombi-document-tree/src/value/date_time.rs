use tombi_ast::TombiValueCommentDirective;

use crate::{
    support::chrono::{
        try_new_local_date, try_new_local_date_time, try_new_local_time, try_new_offset_date_time,
    },
    DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct OffsetDateTime {
    value: tombi_date_time::OffsetDateTime,
    range: tombi_text::Range,
    comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
}

impl OffsetDateTime {
    #[inline]
    pub fn value(&self) -> &tombi_date_time::OffsetDateTime {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDateTime {
    value: tombi_date_time::LocalDateTime,
    range: tombi_text::Range,
    comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct LocalTime {
    value: tombi_date_time::LocalTime,
    range: tombi_text::Range,
    comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
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

impl ValueImpl for OffsetDateTime {
    fn value_type(&self) -> ValueType {
        ValueType::OffsetDateTime
    }

    fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl ValueImpl for LocalDateTime {
    fn value_type(&self) -> ValueType {
        ValueType::LocalDateTime
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
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

impl ValueImpl for LocalTime {
    fn value_type(&self) -> ValueType {
        ValueType::LocalTime
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl From<crate::OffsetDateTime> for tombi_date_time::OffsetDateTime {
    fn from(node: crate::OffsetDateTime) -> Self {
        node.value
    }
}

impl From<crate::LocalDateTime> for tombi_date_time::LocalDateTime {
    fn from(node: crate::LocalDateTime) -> Self {
        node.value
    }
}

impl From<crate::LocalDate> for tombi_date_time::LocalDate {
    fn from(node: crate::LocalDate) -> Self {
        node.value
    }
}

impl From<crate::LocalTime> for tombi_date_time::LocalTime {
    fn from(node: crate::LocalTime) -> Self {
        node.value
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::OffsetDateTime {
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

        match try_new_offset_date_time(&self, toml_version) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::OffsetDateTime(crate::OffsetDateTime {
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

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::LocalDateTime {
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

        match try_new_local_date_time(&self, toml_version) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::LocalDateTime(crate::LocalDateTime {
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

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::LocalTime {
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

        match try_new_local_time(&self, toml_version) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::LocalTime(crate::LocalTime {
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
