use tombi_ast_syntax::{AstNode, TombiValueCommentDirective};

use crate::{DocumentTreeAndErrors, IntoDocumentTreeWithContext, LikeString, ValueImpl, ValueType};

use tombi_document_tree::KeyKind;

#[derive(Debug, Clone)]
pub struct Key<'t> {
    kind: KeyKind,
    pub(crate) value: crate::DocumentText<'t>,
    span: tombi_text::Span,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl<'t> Key<'t> {
    #[inline]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    #[inline]
    pub fn kind(&self) -> KeyKind {
        self.kind
    }

    #[inline]
    pub fn comment_directives(
        &self,
    ) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.comment_directives.as_deref().map(|d| d.iter())
    }

    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.span
    }

    #[inline]
    pub fn unquoted_span(&self) -> tombi_text::Span {
        match self.kind {
            KeyKind::BareKey => self.span,
            KeyKind::BasicString | KeyKind::LiteralString => {
                let mut span = self.span;
                span.start += 1;
                span.end -= 1;
                span
            }
        }
    }
}

impl<'t> PartialEq for Key<'t> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Key<'_> {}

impl<'t> PartialEq<tombi_ast_syntax::Key<'t>> for Key<'t> {
    fn eq(&self, other: &tombi_ast_syntax::Key<'t>) -> bool {
        self.value == other.syntax().text()
    }
}

impl<'t> std::hash::Hash for Key<'t> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<'t> tombi_hashmap::Equivalent<Key<'t>> for &Key<'t> {
    fn equivalent(&self, other: &Key<'t>) -> bool {
        self.value == other.value
    }
}

impl<'t> tombi_hashmap::Equivalent<tombi_ast_syntax::Key<'t>> for &Key<'t> {
    fn equivalent(&self, other: &tombi_ast_syntax::Key<'t>) -> bool {
        **self == *other
    }
}

impl<'t> tombi_hashmap::Equivalent<Key<'t>> for &str {
    fn equivalent(&self, other: &Key<'t>) -> bool {
        self == &other.value
    }
}

impl<'t> tombi_hashmap::Equivalent<Key<'t>> for String {
    #[inline]
    fn equivalent(&self, other: &Key<'t>) -> bool {
        self == other.value.as_str()
    }
}

impl<'t> std::borrow::Borrow<str> for Key<'t> {
    fn borrow(&self) -> &str {
        &self.value
    }
}

impl<'t> std::fmt::Display for Key<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, Option<Key<'t>>> for tombi_ast_syntax::Key<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> crate::DocumentTreeAndErrors<Option<Key<'t>>> {
        let span = self.syntax().span();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: None,
                errors: vec![crate::Error::IncompleteNode { span }],
            };
        };

        let syntax = self.syntax();
        let (value, errors) = match crate::DocumentText::try_new(syntax, context.decoded_text) {
            Ok(value) => (value, Vec::new()),
            Err(error) => (
                crate::DocumentText::new_raw(syntax, context.decoded_text),
                vec![crate::Error::ParseStringError {
                    error,
                    span: self.span(),
                }],
            ),
        };

        let key = Key {
            kind: match self {
                tombi_ast_syntax::Key::BareKey(_) => KeyKind::BareKey,
                tombi_ast_syntax::Key::BasicString(_) => KeyKind::BasicString,
                tombi_ast_syntax::Key::LiteralString(_) => KeyKind::LiteralString,
            },
            value,
            span: token.span(),
            comment_directives: None,
        };

        DocumentTreeAndErrors {
            tree: Some(key),
            errors,
        }
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, Vec<crate::Key<'t>>> for tombi_ast_syntax::Keys<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<Vec<crate::Key<'t>>> {
        let mut keys = Vec::new();
        let mut errors = Vec::new();

        for key in self.keys() {
            let result = key.into_document_tree_with_context(context);
            if !result.errors.is_empty() {
                errors.extend(result.errors);
            }
            if let Some(key) = result.tree {
                keys.push(key);
            }
        }

        DocumentTreeAndErrors { tree: keys, errors }
    }
}

impl<'t> ValueImpl for Key<'t> {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl<'t> LikeString for Key<'t> {
    fn value(&self) -> &str {
        self.value.as_str()
    }

    fn comment_directives(&self) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.comment_directives.as_deref().map(|d| d.iter())
    }
}
