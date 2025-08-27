use tombi_comment_directive::FloatTombiCommentDirective;
use tombi_toml_version::TomlVersion;

use crate::{
    support::float::try_from_float, DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl,
    ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    value: f64,
    range: tombi_text::Range,
    symbol_range: tombi_text::Range,
    comment_directive: Option<Box<FloatTombiCommentDirective>>,
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
        self.range()
    }

    #[inline]
    pub fn comment_directive(&self) -> Option<&FloatTombiCommentDirective> {
        self.comment_directive.as_deref()
    }
}

impl ValueImpl for Float {
    fn value_type(&self) -> ValueType {
        ValueType::Float
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::Float {
    fn into_document_tree_and_errors(
        self,
        _toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        match try_from_float(token.text()) {
            Ok(value) => DocumentTreeAndErrors {
                tree: crate::Value::Float(crate::Float {
                    value,
                    range,
                    symbol_range: range,
                    comment_directive: None,
                }),
                errors: Vec::with_capacity(0),
            },
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseFloatError { error, range }],
            },
        }
    }
}
