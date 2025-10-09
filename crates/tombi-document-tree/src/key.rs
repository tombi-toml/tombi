use tombi_ast::{AstNode, TombiValueCommentDirective};
use tombi_toml_version::TomlVersion;

use crate::{DocumentTreeAndErrors, IntoDocumentTreeAndErrors};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    BareKey,
    BasicString,
    LiteralString,
}

#[derive(Debug, Clone)]
pub struct Key {
    kind: KeyKind,
    pub value: String,
    range: tombi_text::Range,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl Key {
    #[inline]
    pub fn kind(&self) -> KeyKind {
        self.kind
    }

    #[inline]
    pub fn comment_directives(&self) -> Option<&[TombiValueCommentDirective]> {
        self.comment_directives.as_deref()
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

impl PartialEq<tombi_ast::Key> for Key {
    fn eq(&self, other: &tombi_ast::Key) -> bool {
        self.value == other.syntax().text().to_string()
    }
}

impl std::hash::Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl std::borrow::Borrow<String> for Key {
    fn borrow(&self) -> &String {
        &self.value
    }
}

impl indexmap::Equivalent<Key> for &Key {
    fn equivalent(&self, other: &Key) -> bool {
        self.value == other.value
    }
}

impl indexmap::Equivalent<tombi_ast::Key> for &Key {
    fn equivalent(&self, other: &tombi_ast::Key) -> bool {
        self.value == other.syntax().text().to_string()
    }
}

impl indexmap::Equivalent<Key> for &str {
    fn equivalent(&self, other: &Key) -> bool {
        self == &other.value
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

impl IntoDocumentTreeAndErrors<Option<Key>> for tombi_ast::Key {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> crate::DocumentTreeAndErrors<Option<Key>> {
        let range = self.syntax().range();
        let Some(token) = self.token() else {
            return DocumentTreeAndErrors {
                tree: None,
                errors: vec![crate::Error::IncompleteNode { range }],
            };
        };

        // Convert ParseError to crate::Error directly, not via error::Error
        let (value, errors) = match self.try_to_raw_text(toml_version) {
            Ok(value) => (value, Vec::with_capacity(0)),
            Err(error) => (
                token.text().to_string(),
                vec![crate::Error::ParseStringError {
                    error,
                    range: self.range(),
                }],
            ),
        };

        let key = Key {
            kind: match self {
                tombi_ast::Key::BareKey(_) => KeyKind::BareKey,
                tombi_ast::Key::BasicString(_) => KeyKind::BasicString,
                tombi_ast::Key::LiteralString(_) => KeyKind::LiteralString,
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

impl IntoDocumentTreeAndErrors<Vec<crate::Key>> for tombi_ast::Keys {
    fn into_document_tree_and_errors(
        self,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> DocumentTreeAndErrors<Vec<crate::Key>> {
        let mut keys = Vec::new();
        let mut errors = Vec::new();

        for key in self.keys() {
            let result = key.into_document_tree_and_errors(toml_version);
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
