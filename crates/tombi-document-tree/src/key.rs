use tombi_ast::AstNode;
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
    value: String,
    range: tombi_text::Range,
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

impl Key {
    pub fn try_new(
        kind: KeyKind,
        value: String,
        range: tombi_text::Range,
        toml_version: TomlVersion,
    ) -> Result<Self, crate::Error> {
        let key = Self { kind, value, range };
        key.try_to_raw_string(toml_version)?;

        Ok(key)
    }
    pub fn kind(&self) -> KeyKind {
        self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn to_raw_text(&self, toml_version: TomlVersion) -> String {
        // NOTE: Key has already been validated by `impl TryIntoDocumentTree<Key>`,
        //       so it's safe to unwrap.
        self.try_to_raw_string(toml_version).unwrap()
    }

    fn try_to_raw_string(
        &self,
        toml_version: TomlVersion,
    ) -> Result<std::string::String, crate::Error> {
        match self.kind {
            KeyKind::BareKey => tombi_toml_text::try_from_bare_key(&self.value, toml_version),
            KeyKind::BasicString => {
                tombi_toml_text::try_from_basic_string(&self.value, toml_version)
            }
            KeyKind::LiteralString => tombi_toml_text::try_from_literal_string(&self.value),
        }
        .map_err(|error| crate::Error::ParseStringError {
            error,
            range: self.range,
        })
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
        self.try_to_raw_string(TomlVersion::latest())
            == other.try_to_raw_string(TomlVersion::latest())
    }
}

impl Eq for Key {}

impl std::hash::Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.try_to_raw_string(TomlVersion::latest())
            .unwrap_or_else(|_| self.value.to_string())
            .hash(state);
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

        match Key::try_new(
            match self {
                tombi_ast::Key::BareKey(_) => KeyKind::BareKey,
                tombi_ast::Key::BasicString(_) => KeyKind::BasicString,
                tombi_ast::Key::LiteralString(_) => KeyKind::LiteralString,
            },
            token.text().to_string(),
            token.range(),
            toml_version,
        ) {
            Ok(key) => DocumentTreeAndErrors {
                tree: Some(key),
                errors: Vec::with_capacity(0),
            },
            Err(error) => DocumentTreeAndErrors {
                tree: None,
                errors: vec![error],
            },
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

                return DocumentTreeAndErrors { tree: keys, errors };
            }
            if let Some(key) = result.tree {
                keys.push(key);
            }
        }

        DocumentTreeAndErrors { tree: keys, errors }
    }
}
