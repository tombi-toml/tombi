use tombi_toml_version::TomlVersion;

use crate::{DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boolean {
    value: bool,
    node: tombi_ast::Boolean,
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
            tree: crate::Value::Boolean(crate::Boolean { value, node: self }),
            errors: Vec::with_capacity(0),
        }
    }
}
