use tombi_toml_version::TomlVersion;

use crate::{DocumentTreeAndErrors, IntoDocumentTreeAndErrors, ValueImpl, ValueType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringKind {
    BasicString(tombi_ast::BasicString),
    LiteralString(tombi_ast::LiteralString),
    MultiLineBasicString(tombi_ast::MultiLineBasicString),
    MultiLineLiteralString(tombi_ast::MultiLineLiteralString),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct String {
    kind: StringKind,
    value: std::string::String,
}

impl crate::String {
    pub fn try_new(
        kind: StringKind,
        quoted_string: impl Into<std::string::String>,
        toml_version: TomlVersion,
    ) -> Result<Self, tombi_toml_text::ParseError> {
        let quoted_string = quoted_string.into();

        let value = match &kind {
            StringKind::BasicString(_) => {
                tombi_toml_text::try_from_basic_string(&quoted_string, toml_version)
            }
            StringKind::LiteralString(_) => {
                tombi_toml_text::try_from_literal_string(&quoted_string)
            }
            StringKind::MultiLineBasicString(_) => {
                tombi_toml_text::try_from_multi_line_basic_string(&quoted_string, toml_version)
            }
            StringKind::MultiLineLiteralString(_) => {
                tombi_toml_text::try_from_multi_line_literal_string(&quoted_string)
            }
        }?;

        Ok(Self { kind, value })
    }

    #[inline]
    pub fn kind(&self) -> &StringKind {
        &self.kind
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.value
    }

    #[inline]
    pub fn into_value(self) -> std::string::String {
        self.value
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        match self.kind() {
            StringKind::BasicString(node) => node.token(),
            StringKind::LiteralString(node) => node.token(),
            StringKind::MultiLineBasicString(node) => node.token(),
            StringKind::MultiLineLiteralString(node) => node.token(),
        }
        .unwrap()
        .range()
    }

    #[inline]
    pub fn unquoted_range(&self) -> tombi_text::Range {
        match self.kind() {
            StringKind::BasicString(node) => {
                let mut range = node.token().unwrap().range();
                range.start.column += 1;
                range.end.column -= 1;
                range
            }
            StringKind::LiteralString(node) => {
                let mut range = node.token().unwrap().range();
                range.start.column += 1;
                range.end.column -= 1;
                range
            }
            StringKind::MultiLineBasicString(node) => {
                let mut range = node.token().unwrap().range();
                range.start.column += 3;
                range.end.column -= 3;
                range
            }
            StringKind::MultiLineLiteralString(node) => {
                let mut range = node.token().unwrap().range();
                range.start.column += 3;
                range.end.column -= 3;
                range
            }
        }
    }

    #[inline]
    pub fn symbol_range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl ValueImpl for crate::String {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::BasicString {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        match crate::String::try_new(
            StringKind::BasicString(self),
            token.text().to_string(),
            toml_version,
        ) {
            Ok(string) => DocumentTreeAndErrors {
                tree: crate::Value::String(string),
                errors: Vec::with_capacity(0),
            },
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseStringError { error, range }],
            },
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::LiteralString {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        match crate::String::try_new(
            StringKind::LiteralString(self),
            token.text().to_string(),
            toml_version,
        ) {
            Ok(string) => DocumentTreeAndErrors {
                tree: crate::Value::String(string),
                errors: Vec::with_capacity(0),
            },
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseStringError { error, range }],
            },
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::MultiLineBasicString {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        match crate::String::try_new(
            StringKind::MultiLineBasicString(self),
            token.text().to_string(),
            toml_version,
        ) {
            Ok(string) => DocumentTreeAndErrors {
                tree: crate::Value::String(string),
                errors: Vec::with_capacity(0),
            },
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseStringError { error, range }],
            },
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::MultiLineLiteralString {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let range = self.range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        match crate::String::try_new(
            StringKind::MultiLineLiteralString(self),
            token.text().to_string(),
            toml_version,
        ) {
            Ok(string) => DocumentTreeAndErrors {
                tree: crate::Value::String(string),
                errors: Vec::with_capacity(0),
            },
            Err(error) => DocumentTreeAndErrors {
                tree: crate::Value::Incomplete { range },
                errors: vec![crate::Error::ParseStringError { error, range }],
            },
        }
    }
}
