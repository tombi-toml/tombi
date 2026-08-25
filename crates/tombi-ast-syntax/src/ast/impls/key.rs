use std::cmp::Ordering;

use itertools::{EitherOrBoth, Itertools};
use tombi_accessor::Accessor;
use tombi_toml_version::TomlVersion;

use crate::AstNode;

impl crate::Key {
    pub fn token(&self) -> Option<tombi_ast_syntax::SyntaxToken> {
        match self {
            Self::BareKey(key) => key.token(),
            Self::BasicString(key) => key.token(),
            Self::LiteralString(key) => key.token(),
        }
    }

    pub fn accessor(&self, toml_version: TomlVersion) -> Accessor {
        Accessor::Key(self.content_lossy(toml_version))
    }

    pub fn content_lossy(&self, toml_version: TomlVersion) -> String {
        self.try_to_content(toml_version)
            .map(std::borrow::Cow::into_owned)
            .unwrap_or_else(|_| self.syntax().text().to_string())
    }

    pub fn try_to_content(
        &self,
        toml_version: TomlVersion,
    ) -> Result<std::borrow::Cow<'_, str>, tombi_toml_text::ParseError> {
        self.syntax().try_to_content(toml_version)
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            Self::BareKey(key) => key.range(),
            Self::BasicString(key) => key.range(),
            Self::LiteralString(key) => key.range(),
        }
    }
}

impl PartialOrd for crate::Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (
            self.try_to_content(TomlVersion::latest()),
            other.try_to_content(TomlVersion::latest()),
        ) {
            (Ok(a), Ok(b)) => Some(a.cmp(&b)),
            _ => None,
        }
    }
}

impl crate::Keys {
    /// Returns the last dot written in this TOML key path, including an
    /// incomplete path such as `package.`.
    pub fn last_dot(&self) -> Option<tombi_ast_syntax::SyntaxToken> {
        self.syntax()
            .child_elements()
            .filter_map(tombi_ast_syntax::SyntaxElement::into_token)
            .filter(|token| token.kind() == tombi_ast_syntax::SyntaxKind::DOT)
            .last()
    }

    pub fn accessors(&self, toml_version: TomlVersion) -> Vec<Accessor> {
        self.keys()
            .map(|key| key.accessor(toml_version))
            .collect_vec()
    }

    pub fn starts_with(&self, other: &Self) -> bool {
        self.keys()
            .zip_longest(other.keys())
            .all(|pair| match pair {
                EitherOrBoth::Left(_) => true,
                EitherOrBoth::Right(_) => false,
                EitherOrBoth::Both(left, right) => {
                    match (
                        left.try_to_content(TomlVersion::latest()),
                        right.try_to_content(TomlVersion::latest()),
                    ) {
                        (Ok(left), Ok(right)) => left == right,
                        _ => false,
                    }
                }
            })
    }

    pub fn same_as(&self, other: &Self) -> bool {
        self.keys().count() == other.keys().count() && self.starts_with(other)
    }

    pub fn keys_rev(&self) -> impl Iterator<Item = crate::Key> {
        self.keys().collect_vec().into_iter().rev()
    }
}
