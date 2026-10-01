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
pub struct String<'t> {
    kind: StringKind,
    value: crate::DocumentText<'t>,
    span: tombi_text::Span,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl<'t> std::fmt::Display for String<'t> {
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

impl<'t> crate::String<'t> {
    fn new(
        kind: StringKind,
        value: crate::DocumentText<'t>,
        span: tombi_text::Span,
        comment_directives: Option<Vec<TombiValueCommentDirective>>,
    ) -> Self {
        Self {
            kind,
            value,
            span,
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
    pub fn span(&self) -> tombi_text::Span {
        self.span
    }

    #[inline]
    pub fn unquoted_span(&self) -> tombi_text::Span {
        match self.kind() {
            StringKind::BasicString | StringKind::LiteralString => {
                let mut span = self.span;
                span.start += 1;
                span.end -= 1;
                span
            }
            StringKind::MultiLineBasicString | StringKind::MultiLineLiteralString => {
                let mut span = self.span;
                span.start += 3;
                span.end -= 3;
                span
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

impl<'t> ValueImpl for crate::String<'t> {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl<'t> LikeString for crate::String<'t> {
    fn value(&self) -> &str {
        &self.value
    }

    fn comment_directives(&self) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.comment_directives.as_deref().map(|d| d.iter())
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::BasicString<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let token = self.token();
        let span = self.span();

        into_string_and_errors(self, StringKind::BasicString, token, span, context)
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::LiteralString<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let token = self.token();
        let span = self.span();

        into_string_and_errors(self, StringKind::LiteralString, token, span, context)
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>>
    for tombi_ast_syntax::MultiLineBasicString<'t>
{
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let token = self.token();
        let span = self.span();

        into_string_and_errors(self, StringKind::MultiLineBasicString, token, span, context)
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>>
    for tombi_ast_syntax::MultiLineLiteralString<'t>
{
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let token = self.token();
        let span = self.span();

        into_string_and_errors(
            self,
            StringKind::MultiLineLiteralString,
            token,
            span,
            context,
        )
    }
}

fn into_string_and_errors<'t, T: AstNode<'t>>(
    node: T,
    string_kind: StringKind,
    token: Option<tombi_ast_syntax::SyntaxToken<'t>>,
    span: tombi_text::Span,
    context: &crate::DocumentTreeContext<'t>,
) -> DocumentTreeAndErrors<crate::Value<'t>> {
    let (comment_directives, mut errors) = collect_comment_directives_and_errors(&node);

    let Some(token) = token else {
        errors.push(crate::Error::IncompleteNode { span });

        return DocumentTreeAndErrors {
            tree: crate::Value::Incomplete { span },
            errors,
        };
    };

    let value = match crate::DocumentText::try_new(node.syntax(), context.decoded_text) {
        Ok(value) => crate::Value::String(crate::String::new(
            string_kind,
            value,
            token.span(),
            comment_directives,
        )),
        Err(error) => {
            errors.push(crate::Error::ParseStringError { error, span });

            crate::Value::Incomplete { span }
        }
    };

    DocumentTreeAndErrors {
        tree: value,
        errors,
    }
}
