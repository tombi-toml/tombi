use tombi_toml_version::TomlVersion;

use crate::{DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean {
    value: bool,
    node: tombi_ast::Boolean,
    comment_directive: Option<Box<tombi_comment_directive::BooleanTombiCommentDirective>>,
}

impl Boolean {
    #[inline]
    pub fn value(&self) -> bool {
        self.value
    }

    #[inline]
    pub fn node(&self) -> &tombi_ast::Boolean {
        &self.node
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.node.token().unwrap().range()
    }

    #[inline]
    pub fn symbol_range(&self) -> tombi_text::Range {
        self.range()
    }

    #[inline]
    pub fn comment_directive(
        &self,
    ) -> Option<&tombi_comment_directive::BooleanTombiCommentDirective> {
        self.comment_directive.as_ref().map(|c| c.as_ref())
    }
}

impl ValueImpl for Boolean {
    fn value_type(&self) -> ValueType {
        ValueType::Boolean
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::Boolean {
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

        let value = match token.text() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        };

        DocumentTreeAndErrors {
            tree: crate::Value::Boolean(crate::Boolean {
                value,
                node: self,
                comment_directive: None,
            }),
            errors: Vec::with_capacity(0),
        }
    }
}
