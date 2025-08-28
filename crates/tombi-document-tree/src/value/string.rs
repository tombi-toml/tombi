use tombi_ast::{AstNode, TombiValueCommentDirective};
use tombi_toml_text::{
    to_basic_string, to_literal_string, to_multi_line_basic_string, to_multi_line_literal_string,
};
use tombi_toml_version::TomlVersion;

use crate::{
    value::collect_comment_directives_and_errors, DocumentTreeAndErrors, IntoDocumentTreeAndErrors,
    ValueImpl, ValueType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringKind {
    BasicString,
    LiteralString,
    MultiLineBasicString,
    MultiLineLiteralString,
}

#[derive(Debug, Clone, PartialEq)]
pub struct String {
    kind: StringKind,
    value: std::string::String,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
}

impl std::fmt::Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            StringKind::BasicString => write!(f, "{}", to_basic_string(&self.value)),
            StringKind::LiteralString => write!(f, "{}", to_literal_string(&self.value)),
            StringKind::MultiLineBasicString => {
                write!(f, "{}", to_multi_line_basic_string(&self.value))
            }
            StringKind::MultiLineLiteralString => {
                write!(f, "{}", to_multi_line_literal_string(&self.value))
            }
        }
    }
}

impl crate::String {
    fn try_new(
        kind: StringKind,
        quoted_string: impl Into<std::string::String>,
        range: tombi_text::Range,
        toml_version: TomlVersion,
        comment_directives: Option<Box<Vec<TombiValueCommentDirective>>>,
    ) -> Result<Self, tombi_toml_text::ParseError> {
        let quoted_string = quoted_string.into();

        let value = match &kind {
            StringKind::BasicString => {
                tombi_toml_text::try_from_basic_string(&quoted_string, toml_version)?
            }
            StringKind::LiteralString => tombi_toml_text::try_from_literal_string(&quoted_string)?,
            StringKind::MultiLineBasicString => {
                tombi_toml_text::try_from_multi_line_basic_string(&quoted_string, toml_version)?
            }
            StringKind::MultiLineLiteralString => {
                tombi_toml_text::try_from_multi_line_literal_string(&quoted_string)?
            }
        };

        Ok(Self {
            kind,
            value,
            range,
            comment_directives,
        })
    }

    #[inline]
    pub fn kind(&self) -> StringKind {
        self.kind
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.value
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.range
    }

    #[inline]
    pub fn unquoted_range(&self) -> tombi_text::Range {
        match self.kind() {
            StringKind::BasicString | StringKind::LiteralString => {
                let mut range = self.range;
                range.start.column += 1;
                range.end.column -= 1;
                range
            }
            StringKind::MultiLineBasicString | StringKind::MultiLineLiteralString => {
                let mut range = self.range;
                range.start.column += 3;
                range.end.column -= 3;
                range
            }
        }
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

impl ValueImpl for crate::String {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::BasicString {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let token = self.token();
        let range = self.range();

        into_string_and_errors(self, StringKind::BasicString, token, range, toml_version)
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::LiteralString {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let token = self.token();
        let range = self.range();

        into_string_and_errors(self, StringKind::LiteralString, token, range, toml_version)
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::MultiLineBasicString {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let token = self.token();
        let range = self.range();

        into_string_and_errors(
            self,
            StringKind::MultiLineBasicString,
            token,
            range,
            toml_version,
        )
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::MultiLineLiteralString {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let token = self.token();
        let range = self.range();

        into_string_and_errors(
            self,
            StringKind::MultiLineLiteralString,
            token,
            range,
            toml_version,
        )
    }
}

fn into_string_and_errors<T: AstNode>(
    node: T,
    string_kind: StringKind,
    token: Option<tombi_syntax::SyntaxToken>,
    range: tombi_text::Range,
    toml_version: TomlVersion,
) -> DocumentTreeAndErrors<crate::Value> {
    let (comment_directives, mut errors) = collect_comment_directives_and_errors(&node);

    let Some(token) = token else {
        errors.push(crate::Error::IncompleteNode { range });

        return DocumentTreeAndErrors {
            tree: crate::Value::Incomplete { range },
            errors,
        };
    };

    let value = match crate::String::try_new(
        string_kind,
        token.text().to_string(),
        token.range(),
        toml_version,
        comment_directives,
    ) {
        Ok(string) => crate::Value::String(string),
        Err(error) => {
            errors.push(crate::Error::ParseStringError { error, range });

            crate::Value::Incomplete { range }
        }
    };

    DocumentTreeAndErrors {
        tree: value,
        errors,
    }
}
