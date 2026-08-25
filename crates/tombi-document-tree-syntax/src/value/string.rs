use tombi_ast_syntax::{AstNode, TombiValueCommentDirective};
use tombi_toml_text::{
    to_basic_string, to_literal_string, to_multi_line_basic_string, to_multi_line_literal_string,
};

use crate::{
    DocumentTreeAndErrors, IntoDocumentTreeWithContext, LikeString, ValueImpl, ValueType,
    value::collect_comment_directives_and_errors,
};

use tombi_document_tree::StringKind;

#[derive(Debug, Clone, PartialEq)]
pub struct String {
    kind: StringKind,
    value: crate::DocumentText,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
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
    fn new(
        kind: StringKind,
        value: crate::DocumentText,
        range: tombi_text::Range,
        comment_directives: Option<Vec<TombiValueCommentDirective>>,
    ) -> Self {
        Self {
            kind,
            value,
            range,
            comment_directives,
        }
    }

    #[inline]
    pub fn kind(&self) -> StringKind {
        self.kind
    }

    #[inline]
    pub fn value(&self) -> &str {
        self.value.as_str()
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
    pub fn comment_directives(
        &self,
    ) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.comment_directives.as_deref().map(|d| d.iter())
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

impl LikeString for crate::String {
    fn value(&self) -> &str {
        &self.value
    }

    fn comment_directives(&self) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.comment_directives.as_deref().map(|d| d.iter())
    }
}

impl IntoDocumentTreeWithContext<crate::Value> for tombi_ast_syntax::BasicString {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let token = self.token();
        let range = self.range();

        into_string_and_errors(self, StringKind::BasicString, token, range, context)
    }
}

impl IntoDocumentTreeWithContext<crate::Value> for tombi_ast_syntax::LiteralString {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let token = self.token();
        let range = self.range();

        into_string_and_errors(self, StringKind::LiteralString, token, range, context)
    }
}

impl IntoDocumentTreeWithContext<crate::Value> for tombi_ast_syntax::MultiLineBasicString {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let token = self.token();
        let range = self.range();

        into_string_and_errors(
            self,
            StringKind::MultiLineBasicString,
            token,
            range,
            context,
        )
    }
}

impl IntoDocumentTreeWithContext<crate::Value> for tombi_ast_syntax::MultiLineLiteralString {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let token = self.token();
        let range = self.range();

        into_string_and_errors(
            self,
            StringKind::MultiLineLiteralString,
            token,
            range,
            context,
        )
    }
}

fn into_string_and_errors<T: AstNode>(
    node: T,
    string_kind: StringKind,
    token: Option<tombi_ast_syntax::SyntaxToken>,
    range: tombi_text::Range,
    context: &crate::DocumentTreeContext,
) -> DocumentTreeAndErrors<crate::Value> {
    let (comment_directives, mut errors) = collect_comment_directives_and_errors(&node);

    let Some(token) = token else {
        errors.push(crate::Error::IncompleteNode { range });

        return DocumentTreeAndErrors {
            tree: crate::Value::Incomplete { range },
            errors,
        };
    };

    let value = match crate::DocumentText::try_new(node.syntax(), &context.decoded_text) {
        Ok(value) => crate::Value::String(crate::String::new(
            string_kind,
            value,
            token.range(),
            comment_directives,
        )),
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
