use tombi_ast_syntax::{AstNode, TombiValueCommentDirective};

use crate::{DocumentTreeAndErrors, IntoDocumentTreeWithContext, LikeString, ValueImpl, ValueType};

use tombi_document_tree::KeyKind;

#[derive(Debug, Clone)]
pub struct Key {
    kind: KeyKind,
    pub(crate) value: crate::DocumentText,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl Key {
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
    pub fn range(&self) -> tombi_text::Range {
        self.range
    }

    #[inline]
    pub fn unquoted_range(&self) -> tombi_text::Range {
        match self.kind {
            KeyKind::BareKey => self.range,
            KeyKind::BasicString | KeyKind::LiteralString => {
                let mut range = self.range;
                range.start.column += 1;
                range.end.column -= 1;
                range
            }
        }
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Key {}

impl PartialEq<tombi_ast_syntax::Key> for Key {
    fn eq(&self, other: &tombi_ast_syntax::Key) -> bool {
        self.value == other.syntax().text()
    }
}

impl std::hash::Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl tombi_hashmap::Equivalent<Key> for &Key {
    fn equivalent(&self, other: &Key) -> bool {
        self.value == other.value
    }
}

impl tombi_hashmap::Equivalent<tombi_ast_syntax::Key> for &Key {
    fn equivalent(&self, other: &tombi_ast_syntax::Key) -> bool {
        **self == *other
    }
}

impl tombi_hashmap::Equivalent<Key> for &str {
    fn equivalent(&self, other: &Key) -> bool {
        self == &other.value
    }
}

impl tombi_hashmap::Equivalent<Key> for String {
    #[inline]
    fn equivalent(&self, other: &Key) -> bool {
        self == other.value.as_str()
    }
}

impl std::borrow::Borrow<str> for Key {
    fn borrow(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl IntoDocumentTreeWithContext<Option<Key>> for tombi_ast_syntax::Key {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext,
    ) -> crate::DocumentTreeAndErrors<Option<Key>> {
        let range = self.syntax().range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: None,
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        let syntax = self.syntax();
        let (value, errors) = match crate::DocumentText::try_new(syntax, &context.decoded_text) {
            Ok(value) => (value, Vec::new()),
            Err(error) => (
                crate::DocumentText::new_raw(syntax, &context.decoded_text),
                vec![crate::Error::ParseStringError {
                    error,
                    range: self.range(),
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
            range: token.range(),
            comment_directives: None,
        };

        DocumentTreeAndErrors {
            tree: Some(key),
            errors,
        }
    }
}

impl IntoDocumentTreeWithContext<Vec<crate::Key>> for tombi_ast_syntax::Keys {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext,
    ) -> DocumentTreeAndErrors<Vec<crate::Key>> {
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

impl ValueImpl for Key {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl LikeString for Key {
    fn value(&self) -> &str {
        self.value.as_str()
    }

    fn comment_directives(&self) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.comment_directives.as_deref().map(|d| d.iter())
    }
}
